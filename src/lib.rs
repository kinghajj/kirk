//! Pools of workers that perform generic jobs, statically or dynamically.

#![cfg_attr(feature = "nightly",
            feature(recover))]

extern crate crossbeam;
#[macro_use]
extern crate log;
extern crate num_cpus;

use std::mem;
use std::ops::Drop;
use std::thread;

#[cfg(feature = "nightly")]
use std::panic::RecoverSafe;

use crossbeam::Scope;

pub use crew::{Crew, Parameters, Worker};

pub use crew::channel::Channel;
pub use crew::deque::Deque;

pub mod crew;

/// The generic "job" that a pool's workers can perform.
#[cfg(feature = "nightly")]
pub trait Job: Send + RecoverSafe {
    #[inline]
    fn perform(self);
}

/// The generic "job" that a pool's workers can perform.
#[cfg(not(feature = "nightly"))]
pub trait Job: Send {
    #[inline]
    fn perform(self);
}

enum Message<Job> {
    Work(Job),
    Stop,
}

/// A scoped set of worker threads that perform jobs.
///
/// # Scoping
///
/// Because of `crossbeam::scope`, jobs can safely access data on the stack of
/// the original caller, like so:
///
/// ```
/// extern crate crossbeam;
/// extern crate kirk;
///
/// let mut items = [0usize; 8];
/// crossbeam::scope(|scope| {
///     let options = kirk::crew::deque::Options::default();
///     let mut pool = kirk::Pool::<kirk::Deque<kirk::Task>>::scoped(&scope, options);
///     for (i, e) in items.iter_mut().enumerate() {
///         pool.push(move || *e = i)
///     }
/// });
/// ```
///
/// # Worker Details
///
/// Jobs are pushed to workers using the lock-free Chase-Lev deque implemented
/// in `crossbeam`. Each worker runs this loop in a separate thread:
///
///   1. Try to steal a job
///   2. If successful, perform it, and become "hot"
///   3. If shutting down, break
///   4. If lost a race or no jobs, "cool down"
///   5. Possibly yield or sleep
///
/// When a hot worker "cools down," it becomes "warm", with a retry count of
/// zero; an already-warm worker increases the retry count. Eventually, the
/// retry threshold may be exceeded, and the worker becomes "cold".
///
/// The temperature determines the action at step five: a hot worker immediately
/// continues the loop; a warm one cooperatively yields to another thread; and
/// cold ones sleep. The goal is to allow workers to progress unhindered as long
/// as there are jobs, but reduce excessive CPU usage during periods when there
/// are little to none.
///
/// A cold worker may also become hot again if it loses a race to steal a job,
/// since this strongly indicates that another job is ready, but remains cold
/// as long as it finds the queue empty.
pub struct Pool<C>
    where C: Crew
{
    crew: C,
    handles: Option<Vec<(usize, thread::JoinHandle<()>)>>,
}

impl<'scope, C, J> Pool<C>
    where J: Job + 'scope,
          C: Crew<Job = J> + 'scope
{
    /// Create a new scoped worker pool.
    ///
    /// Even after dropped, the workers continue processing outstanding jobs.
    /// The enclosing thread will only block once the `crossbeam::scope` ends,
    /// as it joins on all child threads.
    pub fn scoped(scope: &Scope<'scope>, settings: C::Settings) -> Pool<C> {
        let mut crew = C::new(settings);
        for _ in 0..settings.num_workers() {
            let mut worker = crew.hire();
            scope.spawn(move || {
                worker.run();
            });
        }
        Pool {
            crew: crew,
            handles: None,
        }
    }

    /// Add a job to the pool.
    pub fn push<F>(&mut self, f: F)
        where J: From<F>
    {
        self.crew.give(f);
    }
}

impl<C, J> Pool<C>
    where J: Job + 'static,
          C: Crew<Job = J> + 'static
{
    pub fn new(settings: C::Settings) -> Pool<C> {
        let mut crew = C::new(settings);
        let mut handles = Vec::with_capacity(settings.num_workers());
        for id in 0..settings.num_workers() {
            let mut worker = crew.hire();
            handles.push((id,
                          thread::spawn(move || {
                worker.run();
            })));
        }
        Pool {
            crew: crew,
            handles: Some(handles),
        }
    }
}

// When a pool is dropped, tell each worker to stop.
impl<C> Drop for Pool<C> where C: Crew
{
    fn drop(&mut self) {
        self.crew.stop();
        // if an unscoped pool, await workers
        let mut handles = None;
        mem::swap(&mut self.handles, &mut handles);
        if let Some(handles) = handles {
            for (id, handle) in handles {
                if let Err(e) = handle.join() {
                    error!("while joining worker #{}: {:?}", id, e);
                }
            }
        }
    }
}

// From `scoped_threadpool` crate

trait FnBox {
    #[inline]
    fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
    #[inline]
    fn call_box(self: Box<F>) {
        (*self)()
    }
}

/// A boxed closure that can be performed as a job.
///
/// This allows pools to execute any kind of job, but with the increased cost of
/// dynamic invocation.
///
/// # Warning
///
/// Because of the epoch-based memory management of `crossbeam`, it's possible
/// for `Task` pools to deadlock if the closures own data that cause another
/// thread to block. For example, this would hang if the line `drop(tx);` were
/// removed from the tasks' closure.
///
/// ```
/// extern crate crossbeam;
/// extern crate kirk;
///
/// crossbeam::scope(|scope| {
///     let rx = {
///         let (tx, rx) = std::sync::mpsc::channel();
///         let options = kirk::crew::deque::Options::default();
///         let mut pool = kirk::Pool::<kirk::Deque<kirk::Task>>::scoped(&scope, options);
///         for i in 0..10 {
///             let tx = tx.clone();
///             pool.push(move || {
///                tx.send(i + 1).unwrap();
///                drop(tx);
///             });
///         }
///         rx
///     };
///     scope.spawn(move || {
///         for _ in rx.iter() {}
///     });
/// })
/// ```
pub struct Task<'a>(Box<FnBox + Send + 'a>);

impl<'a> Job for Task<'a> {
    #[inline]
    fn perform(self) {
        let Task(task) = self;
        task.call_box();
    }
}

#[cfg(feature = "nightly")]
impl<'a> RecoverSafe for Task<'a> {}

// Allow closures to be converted to tasks automatically for convenience.
impl<'a, F> From<F> for Task<'a> where F: FnOnce() + Send + 'a
{
    #[inline]
    fn from(f: F) -> Task<'a> {
        Task(Box::new(f))
    }
}

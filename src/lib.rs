//! Pools of generic crews that perform generic jobs, statically or dynamically.
//!
//! # Pools
//!
//! With fewest restrictions, a pool could run any task.
//!
//! ```
//! use kirk::{Pool, Task, Deque};
//! use kirk::crew::deque::Options;
//!
//! let options = Options::default();
//! let mut pool = Pool::<Deque<Task>>::new(options);
//! pool.push(|| { println!("Hello!") });
//! pool.push(|| { println!("World!") });
//! ```
//!
//! # Crews
//!
//! Crews abstract the method of sending jobs to workers. Another implementation
//! is `Channel`.
//!
//! ```
//! use kirk::{Pool, Task, Channel};
//! use kirk::crew::channel::Options;
//!
//! let options = Options::default();
//! let mut pool = Pool::<Channel<Task>>::new(options);
//! pool.push(|| { println!("Hello!") });
//! pool.push(|| { println!("World!") });
//! ```
//!
//! The goal is to support various methods for different workloads, while making
//! it easy to switch from one to another.
//!
//! # Jobs and Tasks
//!
//! The core abstraction is `Job`, which has a single, consuming method,
//! `perform`. Defining a data structure that implements `Job` allows worker
//! threads to statically dispatch the call to `perform`.
//!
//! ```
//! use kirk::{Pool, Job, Deque};
//! use kirk::crew::deque::Options;
//!
//! enum Msg { Hello, World };
//!
//! impl Job for Msg {
//!     fn perform(self) {
//!         match self {
//!             Msg::Hello => println!("Hello!"),
//!             Msg::World => println!("World!"),
//!         }
//!     }
//! }
//!
//! let options = Options::default();
//! let mut pool = Pool::<Deque<Msg>>::new(options);
//! pool.push(Msg::Hello);
//! pool.push(Msg::World);
//! ```
//!
//! As shown before, however, there's a predefined type `Task`, a boxed closure.
//! This allows arbitary jobs to be pushed to a pool, but with added costs both
//! to construct the box and to call each dynamically.
//!
//! # Scoping
//!
//! Using `crossbeam::scope`, jobs can safely access data on the stack of the
//! original caller, like so:
//!
//! ```
//! extern crate crossbeam;
//! extern crate kirk;
//!
//! let mut items = [0usize; 8];
//! crossbeam::scope(|scope| {
//!     let options = kirk::crew::deque::Options::default();
//!     let mut pool = kirk::Pool::<kirk::Deque<kirk::Task>>::scoped(&scope, options);
//!     for (i, e) in items.iter_mut().enumerate() {
//!         pool.push(move || *e = i)
//!     }
//! });
//! ```

#![cfg_attr(feature = "nightly",
            feature(recover))]

#![cfg_attr(feature = "plugins",
            feature(plugin))]

#![cfg_attr(feature = "clippy",
            plugin(clippy))]

extern crate crossbeam;
#[macro_use]
extern crate log;
extern crate num_cpus;

use std::ops::Drop;
use std::thread;

#[cfg(feature = "nightly")]
use std::panic::RecoverSafe;

use crossbeam::Scope;

pub use crew::{Crew, Parameters, Worker};
pub use crew::channel::Channel;
pub use crew::deque::Deque;

pub mod crew;

/// An item of work to be executed in a thread pool.
#[cfg(feature = "nightly")]
pub trait Job: Send + RecoverSafe {
    /// Consume and execute the job.
    #[inline]
    fn perform(self);
}

/// An item of work to be executed in a thread pool.
#[cfg(not(feature = "nightly"))]
pub trait Job: Send {
    /// Consume and execute the job.
    #[inline]
    fn perform(self);
}

enum Message<Job> {
    Work(Job),
    Stop,
}

/// A crew of workers that perform jobs on separate threads.
///
/// Even after dropped, the workers continue processing outstanding jobs, then
/// halt when there are no more. Any shared resources of the crew would get
/// dropped then.
///
/// If scoped, the enclosing thread will only block once the `crossbeam::scope`
/// ends, since it must join on all threads it has spawned for safety.
pub struct Pool<C>
    where C: Crew
{
    crew: C,
}

impl<'scope, C, J> Pool<C>
    where J: Job + 'scope,
          C: Crew<Job = J> + 'scope
{
    /// Create a new, scoped pool.
    pub fn scoped(scope: &Scope<'scope>, settings: C::Settings) -> Pool<C> {
        let mut crew = C::new(settings);
        for _ in 0..settings.num_workers() {
            let mut worker = crew.hire();
            scope.spawn(move || {
                worker.run();
            });
        }
        Pool { crew: crew }
    }

    /// Give a job to the pool's crew.
    pub fn push<F>(&mut self, f: F)
        where J: From<F>
    {
        self.crew.give(f);
    }
}

// Hello!

impl<C, J> Pool<C>
    where J: Job + 'static,
          C: Crew<Job = J> + 'static
{
    /// Create an new, unscoped pool.
    ///
    /// When dropped, each worker thread is joined, to ensure that all
    /// outstanding jobs
    pub fn new(settings: C::Settings) -> Pool<C> {
        let mut crew = C::new(settings);
        for _ in 0..settings.num_workers() {
            let mut worker = crew.hire();
            thread::spawn(move || {
                worker.run();
            });
        }
        Pool { crew: crew }
    }
}

// When a pool is dropped, tell each worker to stop.
impl<C> Drop for Pool<C> where C: Crew
{
    fn drop(&mut self) {
        self.crew.stop();
    }
}

// From `scoped_threadpool` crate

trait FnBox {
    #[inline]
    fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
    #[cfg_attr(feature = "clippy",
               allow(boxed_local))]
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
/// use std::sync::mpsc::channel;
/// use std::thread;
/// use kirk::{Pool, Task, Deque};
/// use kirk::crew::deque::Options;
///
/// let rx = {
///     let (tx, rx) = channel();
///     let options = Options::default();
///     let mut pool = Pool::<Deque<Task>>::new(options);
///     for i in 0..10 {
///         let tx = tx.clone();
///         pool.push(move || {
///            tx.send(i + 1).unwrap();
///            drop(tx);
///         });
///     }
///     rx
/// };
/// thread::spawn(move || {
///     for _ in rx.iter() {}
/// });
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

//! Pools of workers that perform generic jobs, statically or dynamically.

#![cfg_attr(feature = "nightly",
            feature(recover))]

extern crate crossbeam;
#[macro_use]
extern crate log;
extern crate num_cpus;

use std::default::Default;
use std::mem;
use std::ops::Drop;
use std::thread;
use std::thread::{sleep, yield_now};
use std::time::Duration;

#[cfg(feature = "nightly")]
use std::panic::{recover, RecoverSafe};

use crossbeam::Scope;
use crossbeam::sync::chase_lev;
use crossbeam::sync::chase_lev::Steal::{Data, Abort, Empty};

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

enum Load {
    Hot,
    Warm,
    Cold,
}

struct Worker {
    #[cfg_attr(not(feature = "nightly"), allow(dead_code))]
    id: usize,
    load: Load,
    retries: u32,
    options: Options,
}

impl Worker {
    fn new(id: usize, options: Options) -> Worker {
        Worker {
            id: id,
            load: Load::Hot,
            retries: 0,
            options: options,
        }
    }

    #[inline]
    fn run<J: Job>(&mut self, stealer: chase_lev::Stealer<Message<J>>) {
        loop {
            match stealer.steal() {
                Data(Message::Work(job)) => self.does(job),
                Data(Message::Stop) => break,
                Abort => self.missed(),
                Empty => self.nothing(),
            }
            self.wait();
        }
    }

    // the worker just successfully acquired an item
    // this version uses `recover` to handle panics from jobs
    #[cfg(feature = "nightly")]
    #[inline]
    fn does<J: Job>(&mut self, job: J) {
        recover(|| {
            job.perform();
        })
            .map_err(|e| error!("worker #{}: job panicked: {:?}", self.id, e))
            .ok();
        self.load = Load::Hot;
    }

    #[cfg(not(feature = "nightly"))]
    // the worker just successfully acquired an item
    // this version propogates panics from job
    #[inline]
    fn does<J: Job>(&mut self, job: J) {
        job.perform();
        self.load = Load::Hot;
    }

    // the worker just lost a race to acquire an item
    #[inline]
    fn missed(&mut self) {
        if self.options.retry_threshold == 0 {
            self.load = Load::Cold;
            return;
        }
        match self.load {
            Load::Hot => self.become_warm(),
            Load::Warm => self.become_cooler(),
            Load::Cold => self.load = Load::Hot,
        }
    }

    // the worker just found an empty work queue
    #[inline]
    fn nothing(&mut self) {
        if self.options.retry_threshold == 0 {
            self.load = Load::Cold;
            return;
        }
        match self.load {
            Load::Hot => self.become_warm(),
            Load::Warm => self.become_cooler(),
            Load::Cold => {}
        }
    }

    // continue, yield, or sleep based on the load
    #[inline]
    fn wait(&self) {
        match self.load {
            Load::Hot => {}
            Load::Warm => yield_now(),
            Load::Cold => sleep(self.options.cold_interval),
        }
    }

    #[inline]
    fn become_warm(&mut self) {
        self.load = Load::Warm;
        self.retries = 0;
    }

    #[inline]
    fn become_cooler(&mut self) {
        self.retries += 1;
        // exceeded threshold, become cold
        if self.retries >= self.options.retry_threshold {
            self.load = Load::Cold;
        }
    }
}

/// Parameters to adjust the size and behavior of a pool.
///
/// The current defaults for retry threshold and cold interval--32 and 1ms--were
/// chosen arbitrarily. Experimentation may be prudent.
#[derive(Copy, Clone)]
pub struct Options {
    /// How many times may a worker fail to acquire a job before it becomes
    /// "cold" and sleeps for `cold_interval` between subsequent attempts.
    pub retry_threshold: u32,
    /// The minimum length of time a worker will sleep when it is cold.
    pub cold_interval: Duration,
    /// The number of workers to create in the pool.
    pub num_workers: usize,
}

impl Default for Options {
    fn default() -> Options {
        Options {
            retry_threshold: 32,
            cold_interval: Duration::from_millis(1),
            num_workers: num_cpus::get(),
        }
    }
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
///     let mut pool = kirk::Pool::<kirk::Task>::scoped(&scope, kirk::Options::default());
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
pub struct Pool<J> {
    options: Options,
    sender: chase_lev::Worker<Message<J>>,
    handles: Option<Vec<(usize, thread::JoinHandle<()>)>>,
}

impl<'scope, J: Job + 'scope> Pool<J> {
    /// Create a new scoped worker pool.
    ///
    /// Even after dropped, the workers continue processing outstanding jobs.
    /// The enclosing thread will only block once the `crossbeam::scope` ends,
    /// as it joins on all child threads.
    pub fn scoped(scope: &Scope<'scope>, options: Options) -> Pool<J> {
        let (sender, stealer) = chase_lev::deque();
        for id in 0..options.num_workers {
            let stealer = stealer.clone();
            let mut worker = Worker::new(id, options);
            scope.spawn(move || {
                worker.run(stealer);
            });
        }
        Pool {
            options: options,
            sender: sender,
            handles: None,
        }
    }

    /// Add a job to the pool.
    pub fn push<F>(&mut self, f: F)
        where J: From<F>
    {
        self.sender.push(Message::Work(J::from(f)));
    }
}

impl<J: Job + 'static> Pool<J> {
    /// Create a new unscoped worker pool.
    ///
    /// When dropped, such a pool joins on all worker threads, which continue
    /// processing outstanding jobs.
    pub fn new(options: Options) -> Pool<J> {
        let (sender, stealer) = chase_lev::deque();
        let mut handles = Vec::with_capacity(options.num_workers);
        for id in 0..options.num_workers {
            let stealer = stealer.clone();
            let mut worker = Worker::new(id, options);
            let handle = thread::spawn(move || {
                worker.run(stealer);
            });
            handles.push((id, handle));
        }
        Pool {
            options: options,
            sender: sender,
            handles: Some(handles),
        }
    }
}

// When a pool is dropped, tell each worker to stop.
impl<J> Drop for Pool<J> {
    fn drop(&mut self) {
        for _ in 0..self.options.num_workers {
            self.sender.push(Message::Stop);
        }
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
///         let mut pool = kirk::Pool::<kirk::Task>::scoped(&scope, kirk::Options::default());
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

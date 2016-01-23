//! A scoped task pool using `crossbeam`.
//!
//! This is similar to `scoped_threadpool`, except that it uses the lock-free
//! deque from `crossbeam`. Adding tasks is cheap:  on my MacBook, the benchmark
//! `enqueue_nop_task` takes 75 ns/iter. The intended use case is when finding
//! tasks is I/O-bound, while the tasks are CPU-bound. Because the deque is
//! work-stealing, threads always stay busy as long as there are outstanding
//! tasks. However, to avoid excessive CPU usage when there are no tasks, the
//! threads will "cool down" and sleep between checking the deque.
//!
//! Currently, this makes use of the unstable "recover" feature to prevent
//! panics in tasks causing the entire pool to come down. It should be possible
//! to use conditional compilation to let the crate compile in stable Rust, just
//! without the ability to recover from panics.

// Not working... why?
// #![cfg_attr(feature = "nightly",
//             feature(recover))]

#![feature(recover, stmt_expr_attributes)]

extern crate crossbeam;
#[macro_use]
extern crate log;
extern crate num_cpus;

use std::default::Default;
#[cfg(tracing)]
use std::mem;
use std::ops::Drop;
use std::panic::{recover, RecoverSafe};
use std::thread::{sleep, yield_now};
use std::time::Duration;

use crossbeam::Scope;
use crossbeam::sync::chase_lev;
use crossbeam::sync::chase_lev::Steal::{Data, Abort, Empty};

// From `scoped_threadpool` crate

trait FnBox {
    fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
    fn call_box(self: Box<F>) {
        (*self)()
    }
}

// Not exactly sure why `Sync` is required, when `scoped_threadpool` did not.
// I think it's due to a possibly-overzealous requirement in `crossbeam`.
// An unfortunate side-effect of this is that channel senders cannot be moved
// into a task.
struct Task<'a>(Box<FnBox + Send + Sync + 'a>);

impl<'a> Task<'a> {
    fn call(self) {
        let Task(task) = self;
        task.call_box();
    }
}

impl<'a> RecoverSafe for Task<'a> {}

enum Message<'a> {
    Work(Task<'a>),
    Stop,
}

/// Each worker thread can be in one of three states--"hot," "warm," and
/// "cold"--that indicate recent activity. A worker that has just successfully
/// stolen a task is always hot. If one encounteres an empty queue or loses
/// a race to steal, it becomes warm, with a retry count of zero; if it was
/// already warm, the count is incremented. Eventually, the retry count may
/// exceed the configured threshold, making the worker become cold. A cold
/// worker may also become hot again if it loses a race to steal a task,
/// since this strongly indicates that another task is ready for execution.
///
/// This determines whether a worker continues immediately, cooperatively yields
/// to another thread, or sleeps before attempting to steal another task,
/// respectfully. The goal is to allow workers to progress unhindered as long
/// as there are tasks to execute, but reduce excessive CPU usage during periods
/// where there are little to none.
enum Load {
    Hot,
    Warm,
    Cold,
}

struct Worker {
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

    // the worker just successfully acquired an item
    fn does<'scope>(&mut self, task: Task<'scope>) {
        recover(|| {
            #[cfg(tracing)]
            trace!("worker #{}: calling task: {:x}",
                   self.id,
                   unsafe { mem::transmute_copy::<Task<'scope>, usize>(&task) });
            task.call();
        })
            .map_err(|e| error!("worker #{}: task panicked: {:?}", self.id, e))
            .ok();
        self.load = Load::Hot;
    }

    // the worker just lost a race to acquire an item
    fn missed(&mut self) {
        trace!("worker #{}: lost race", self.id);
        if self.options.retry_threshold == 0 {
            trace!("worker #{}: become cold", self.id);
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
    fn nothing(&mut self) {
        #[cfg(tracing)]
        trace!("worker #{}: empty queue", self.id);
        if self.options.retry_threshold == 0 {
            #[cfg(tracing)]
            trace!("worker #{}: become cold", self.id);
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
    fn wait(&self) {
        match self.load {
            Load::Hot => {}
            Load::Warm => yield_now(),
            Load::Cold => sleep(self.options.cold_interval),
        }
    }

    #[inline]
    fn become_warm(&mut self) {
        #[cfg(tracing)]
        trace!("worker #{}: become warm", self.id);
        self.load = Load::Warm;
        self.retries = 0;
    }

    #[inline]
    fn become_cooler(&mut self) {
        self.retries += 1;
        #[cfg(tracing)]
        trace!("worker #{}: {} {}",
               self.id,
               self.retries,
               if self.retries == 1 {
                   "retry"
               } else {
                   "retries "
               });
        // exceeded threshold, become cold
        if self.retries >= self.options.retry_threshold {
            #[cfg(tracing)]
            trace!("worker #{}: become cold", self.id);
            self.load = Load::Cold;
        }
    }
}

/// Parameters to adjust the size and behavior of a pool.
#[derive(Copy, Clone)]
pub struct Options {
    /// How many times may a worker fail to acquire a task before it becomes
    /// "cold" and sleeps for `cold_interval` between subsequent attempts.
    pub retry_threshold: u32,
    /// The minimum length of time a worker will sleep when it is cold.
    pub cold_interval: Duration,
    /// The number of workers to create in the pool.
    pub num_workers: usize,
}

impl Default for Options {
    fn default() -> Options {
        // these values were not chosen for any particular reason;
        // benchmarking different configurations would be prudent.
        Options {
            retry_threshold: 32,
            cold_interval: Duration::from_millis(1),
            num_workers: num_cpus::get(),
        }
    }
}

/// A pool of worker threads that execute jobs within a bounded scope.
///
/// Because of `crossbeam::scope`, tasks can safely access data on the stack of
/// the original caller, like so:
///
/// ```
/// extern crate crossbeam;
/// extern crate kirk;
///
/// let mut items = [0usize; 8];
/// crossbeam::scope(|scope| {
///     let mut pool = kirk::Pool::new(&scope, kirk::Options::default());
///     for (i, e) in items.iter_mut().enumerate() {
///         pool.execute(move || *e = i)
///     }
/// });
/// ```
pub struct Pool<'scope> {
    options: Options,
    sender: chase_lev::Worker<Message<'scope>>,
}

impl<'scope> Pool<'scope> {
    /// Create a new task pool.
    pub fn new(scope: &Scope<'scope>, options: Options) -> Pool<'scope> {
        #[cfg(tracing)]
        trace!("creating pool with {} {}",
               options.num_workers,
               if options.num_workers == 1 {
                   "worker"
               } else {
                   "workers"
               });

        let (sender, stealer) = chase_lev::deque();
        for id in 0..options.num_workers {
            let stealer = stealer.clone();
            let mut worker = Worker::new(id, options);
            #[cfg(tracing)]
            trace!("spawning worker #{}", worker.id);
            scope.spawn(move || {
                #[cfg(tracing)]
                trace!("worker #{} running", worker.id);
                loop {
                    match stealer.steal() {
                        Data(Message::Work(task)) => worker.does(task),
                        Data(Message::Stop) => break,
                        Abort => worker.missed(),
                        Empty => worker.nothing(),
                    }
                    worker.wait();
                }
                #[cfg(tracing)]
                trace!("worker #{} stopping", worker.id);
            });
        }
        Pool {
            options: options,
            sender: sender,
        }
    }

    /// Add a task to the pool.
    pub fn execute<F>(&mut self, f: F)
        where F: FnOnce() + Send + Sync + 'scope
    {
        let task = Task(Box::new(f));
        #[cfg(tracing)]
        trace!("adding task: {:x}",
               unsafe { mem::transmute_copy::<Task<'scope>, usize>(&task) });
        self.sender.push(Message::Work(task));
    }
}

// When a pool is dropped, tell each worker to stop.
impl<'scope> Drop for Pool<'scope> {
    fn drop(&mut self) {
        #[cfg(tracing)]
        trace!("pool dropped, telling workers to stop");
        for _ in 0..self.options.num_workers {
            self.sender.push(Message::Stop);
        }
    }
}

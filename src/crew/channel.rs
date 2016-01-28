//! A crew that uses `std::sync::mpsc`.

use std::default::Default;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver};

#[cfg(feature = "nightly")]
use std::panic::recover;

use num_cpus;

use {Job, Message};
use super::{Crew, Parameters, Worker};

/// The `Crew` `Member` for `Channel`.
pub struct ChannelWorker<J> {
    #[cfg_attr(not(feature = "nightly"), allow(dead_code))]
    id: usize,
    receiver: Arc<Mutex<Receiver<Message<J>>>>,
}

impl<J: Job> ChannelWorker<J> {
    // the worker just successfully acquired an item
    // this version uses `recover` to handle panics from jobs
    #[cfg(feature = "nightly")]
    #[inline]
    fn does(&mut self, job: J) {
        recover(|| {
            job.perform();
        })
            .map_err(|e| error!("worker #{}: job panicked: {:?}", self.id, e))
            .ok();
    }

    #[cfg(not(feature = "nightly"))]
    // the worker just successfully acquired an item
    // this version propogates panics from job
    #[inline]
    fn does(&mut self, job: J) {
        job.perform();
    }
}

impl<J: Job> Worker for ChannelWorker<J> {
    fn run(&mut self) {
        loop {
            let message = {
                let lock = self.receiver.lock().unwrap();
                lock.recv()
            };

            match message {
                Ok(Message::Work(job)) => self.does(job),
                _ => break,
            }
        }
    }
}


/// Parameters to adjust the size of the crew.
#[derive(Copy, Clone)]
pub struct Options {
    /// The number of workers to create in the pool.
    pub num_workers: usize,
}

impl Parameters for Options {
    fn num_workers(&self) -> usize {
        self.num_workers
    }
}

impl Default for Options {
    fn default() -> Options {
        Options { num_workers: num_cpus::get() }
    }
}

/// A `Crew` that uses a shared, mutex-wrapped receiver.
pub struct Channel<J> {
    next_id: usize,
    options: Options,
    sender: Sender<Message<J>>,
    receiver: Arc<Mutex<Receiver<Message<J>>>>,
}

impl<J: Job> Crew for Channel<J> {
    type Job = J;
    type Member = ChannelWorker<J>;
    type Settings = Options;

    fn new(options: Options) -> Channel<J> {
        let (tx, rx) = channel();
        Channel {
            next_id: 0,
            options: options,
            sender: tx,
            receiver: Arc::new(Mutex::new(rx)),
        }
    }

    fn hire(&mut self) -> ChannelWorker<J> {
        let id = self.next_id;
        self.next_id += 1;
        ChannelWorker {
            id: id,
            receiver: self.receiver.clone(),
        }
    }

    fn give<F>(&mut self, f: F)
        where J: From<F>
    {
        self.sender.send(Message::Work(J::from(f))).unwrap();
    }

    fn stop(&mut self) {
        for _ in 0..self.options.num_workers {
            self.sender.send(Message::Stop).unwrap();
        }
    }
}

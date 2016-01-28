//! Traits that abstract how to send jobs to workers.

use Job;

pub mod channel;
pub mod deque;

/// A collection of workers that can be given jobs.
pub trait Crew {
    type Job: Job;
    type Member: Worker;
    type Settings: Parameters;

    /// Construct a crew from its parameters.
    fn new(settings: Self::Settings) -> Self;

    /// Construct a new worker.
    fn hire(&mut self) -> Self::Member;

    /// Send a job to the workers.
    fn give<F>(&mut self, job: F) where Self::Job: From<F>;

    /// Tell the workers to stop after completing all outstanding jobs.
    fn stop(&mut self);
}

/// Workers perform jobs in separate threads as part of a crew.
///
/// `Pool` moves each `Worker` into its own thread, which is why they must
/// be `Send`.
pub trait Worker: Send {
    /// Execute the worker's main loop.
    ///
    /// Typecially, this will look somewhat like:
    ///
    ///   1. Possibly acquire a message.
    ///   2. If successful, perform it.
    ///   3. If error or message is 'stop', break.
    ///
    /// Each crew implements these differently, but all fulfill this contract.
    fn run(&mut self);
}

/// Options used to alter the behavior or configuration of a crew.
pub trait Parameters: Copy + Clone {
    /// The number of workers in the crew.
    fn num_workers(&self) -> usize;
}

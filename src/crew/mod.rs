use Job;

pub mod deque;

pub trait Crew {
    type Job: Job;
    type Member: Worker;
    type Settings: Parameters;

    fn new(settings: Self::Settings) -> Self;

    fn hire(&mut self) -> Self::Member;

    fn give<F>(&mut self, job: F) where Self::Job: From<F>;

    fn stop(&mut self);
}

pub trait Worker: Send {
    fn run(&mut self);
}

pub trait Parameters: Copy + Clone {
    fn num_workers(&self) -> usize;
}

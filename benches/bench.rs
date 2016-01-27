#![feature(test, recover)]

extern crate kirk;
extern crate crossbeam;
extern crate test;

use std::panic::RecoverSafe;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use test::{Bencher, black_box};

use kirk::{Job, Pool, Task, Deque, Channel};

struct NopJob;

impl Job for NopJob {
    fn perform(self) {
    }
}

struct AtomicJob(Arc<AtomicUsize>);

impl RecoverSafe for AtomicJob {}

impl Job for AtomicJob {
    fn perform(self) {
        let AtomicJob(counter) = self;
        black_box(counter.fetch_add(1, Ordering::Relaxed));
    }
}

#[inline]
fn fib(n: u64) -> u64 {
    (0..n).fold((0, 1), |(a, b), _| (b, a + b)).0
}

struct FibJob(u64);

impl Job for FibJob {
    fn perform(self) {
        let FibJob(n) = self;
        black_box(fib(n));
    }
}

#[bench]
fn startup_and_teardown(mut b: &mut Bencher) {
    let mut options = kirk::crew::deque::Options::default();
    options.num_workers = 1;
    b.iter(|| {
        crossbeam::scope(|scope| {
            let _ = Pool::<Deque<Task>>::scoped(&scope, options);
        });
    });
}

#[bench]
fn startup_and_teardown_channel(mut b: &mut Bencher) {
    let mut options = kirk::crew::channel::Options::default();
    options.num_workers = 1;
    b.iter(|| {
        crossbeam::scope(|scope| {
            let _ = Pool::<Channel<Task>>::scoped(&scope, options);
        });
    });
}

#[bench]
fn enqueue_nop_job(b: &mut Bencher) {
    let mut options = kirk::crew::deque::Options::default();
    options.num_workers = 1;
    crossbeam::scope(|scope| {
        let mut pool = Pool::<Deque<NopJob>>::scoped(&scope, options);
        b.iter(|| pool.push(NopJob));
    });
}

#[bench]
fn enqueue_nop_job_channel(b: &mut Bencher) {
    let mut options = kirk::crew::channel::Options::default();
    options.num_workers = 1;
    crossbeam::scope(|scope| {
        let mut pool = Pool::<Channel<NopJob>>::scoped(&scope, options);
        b.iter(|| pool.push(NopJob));
    });
}

#[bench]
fn enqueue_nop_task(b: &mut Bencher) {
    let mut options = kirk::crew::deque::Options::default();
    options.num_workers = 1;
    crossbeam::scope(|scope| {
        let mut pool = Pool::<Deque<Task>>::scoped(&scope, options);
        b.iter(|| {
            pool.push(move || {
                black_box(0);
            });
        })
    });
}

#[bench]
fn enqueue_nop_task_channel(b: &mut Bencher) {
    let mut options = kirk::crew::channel::Options::default();
    options.num_workers = 1;
    crossbeam::scope(|scope| {
        let mut pool = Pool::<Channel<Task>>::scoped(&scope, options);
        b.iter(|| {
            pool.push(move || {
                black_box(0);
            });
        })
    });
}

#[bench]
fn enqueue_atomic_task(b: &mut Bencher) {
    let mut options = kirk::crew::deque::Options::default();
    options.num_workers = 1;
    let counter = Arc::new(AtomicUsize::new(0));
    crossbeam::scope(|scope| {
        let mut pool = Pool::<Deque<Task>>::scoped(&scope, options);
        b.iter(|| {
            let counter = counter.clone();
            pool.push(move || {
                black_box(counter.fetch_add(1, Ordering::Relaxed));
            });
        })
    });
}

#[bench]
fn enqueue_atomic_task_channel(b: &mut Bencher) {
    let mut options = kirk::crew::channel::Options::default();
    options.num_workers = 1;
    let counter = Arc::new(AtomicUsize::new(0));
    crossbeam::scope(|scope| {
        let mut pool = Pool::<Channel<Task>>::scoped(&scope, options);
        b.iter(|| {
            let counter = counter.clone();
            pool.push(move || {
                black_box(counter.fetch_add(1, Ordering::Relaxed));
            });
        })
    });
}

#[bench]
fn enqueue_atomic_job(b: &mut Bencher) {
    let mut options = kirk::crew::deque::Options::default();
    options.num_workers = 1;
    let counter = Arc::new(AtomicUsize::new(0));
    crossbeam::scope(|scope| {
        let mut pool = Pool::<Deque<AtomicJob>>::scoped(&scope, options);
        b.iter(|| {
            pool.push(AtomicJob(counter.clone()));
        })
    });
}

#[bench]
fn enqueue_atomic_job_channel(b: &mut Bencher) {
    let mut options = kirk::crew::channel::Options::default();
    options.num_workers = 1;
    let counter = Arc::new(AtomicUsize::new(0));
    crossbeam::scope(|scope| {
        let mut pool = Pool::<Channel<AtomicJob>>::scoped(&scope, options);
        b.iter(|| {
            pool.push(AtomicJob(counter.clone()));
        })
    });
}

#[bench]
fn enqueue_fib_task(b: &mut Bencher) {
    let options = kirk::crew::deque::Options::default();
    crossbeam::scope(|scope| {
        let mut pool = Pool::<Deque<Task>>::scoped(&scope, options);
        b.iter(|| {
            pool.push(move || {
                black_box(fib(1000000));
            });
        });
    });
}

#[bench]
fn enqueue_fib_task_channel(b: &mut Bencher) {
    let options = kirk::crew::channel::Options::default();
    crossbeam::scope(|scope| {
        let mut pool = Pool::<Channel<Task>>::scoped(&scope, options);
        b.iter(|| {
            pool.push(move || {
                black_box(fib(1000000));
            });
        });
    });
}

#[bench]
fn enqueue_fib_job(b: &mut Bencher) {
    let options = kirk::crew::deque::Options::default();
    crossbeam::scope(|scope| {
        let mut pool = Pool::<Deque<FibJob>>::scoped(&scope, options);
        b.iter(|| {
            pool.push(black_box(FibJob(1000000)));
        });
    });
}

#[bench]
fn enqueue_fib_job_channel(b: &mut Bencher) {
    let options = kirk::crew::channel::Options::default();
    crossbeam::scope(|scope| {
        let mut pool = Pool::<Channel<FibJob>>::scoped(&scope, options);
        b.iter(|| {
            pool.push(black_box(FibJob(1000000)));
        });
    });
}

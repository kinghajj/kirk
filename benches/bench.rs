#![feature(test)]

extern crate kirk;
extern crate crossbeam;
extern crate test;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use test::{Bencher, black_box};

use kirk::{Options, Pool};

#[bench]
fn startup_and_teardown(mut b: &mut Bencher) {
    let mut options = Options::default();
    options.num_workers = 1;
    b.iter(|| {
        crossbeam::scope(|scope| {
            let _ = Pool::new(&scope, options);
        });
    });
}

#[bench]
fn enqueue_nop_task(b: &mut Bencher) {
    let mut options = Options::default();
    options.num_workers = 1;
    crossbeam::scope(|scope| {
        let mut pool = Pool::new(&scope, options);
        b.iter(|| {
            pool.execute(move || {
                black_box(0);
            });
        })
    });
}

#[bench]
fn enqueue_atomic_task(b: &mut Bencher) {
    let mut options = Options::default();
    options.num_workers = 1;
    let counter = Arc::new(AtomicUsize::new(0));
    crossbeam::scope(|scope| {
        let mut pool = Pool::new(&scope, options);
        b.iter(|| {
            let counter = counter.clone();
            pool.execute(move || {
                black_box(counter.fetch_add(1, Ordering::Relaxed));
            });
        })
    });
}

fn fib(n: u64) -> u64 {
    (0..n).fold((0, 1), |(a, b), _| (b, a + b)).0
}

#[bench]
fn enqueue_fib_task(b: &mut Bencher) {
    let options = Options::default();
    crossbeam::scope(|scope| {
        let mut pool = Pool::new(&scope, options);
        b.iter(|| {
            pool.execute(move || {
                black_box(fib(1000000));
            });
        });
    });
}

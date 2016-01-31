# `kirk`, a highly-flexible thread pool for Rust

[![Build Status](https://travis-ci.org/kinghajj/kirk.svg?branch=master)](https://travis-ci.org/kinghajj/kirk) [![Coverage Status](https://coveralls.io/repos/github/kinghajj/kirk/badge.svg?branch=master)](https://coveralls.io/github/kinghajj/kirk?branch=master)

## Documentation

The [documentation][docs] is hosted on GitHub Pages.

[docs]: http://kinghajj.github.io/kirk/kirk/

## Stability

The API and internal designs are subject to rapid change, so breakages could
happen frequently until a pleasant and high-performance interface is found.
Play around with this and figure out ways to improve it, but don't rely on it
yet for anything truly important.

## Examples

Pools are generic not only to the jobs performed, but also to the method for
giving jobs to workers. They may also be scoped, allowing safe access to data
in the stack of the originating thread.

```rust
// unscoped, dynamic dispatch using a Chase-Lev deque
let mut pool1 = Pool::<Deque<Task>>::new(deque::Options::default());
pool1.push(|| { println!("Hello!") });
pool1.push(|| { println!("World!") });

// unscoped, dynamic dispatch using a locked channel
let mut pool2 = Pool::<Channel<Task>>::new(channel::Options::default());
pool2.push(|| { println!("Hello!") });
pool2.push(|| { println!("World!") });

// unscoped, static dispatch
enum Msg { Hello, World };
impl Job for Msg {
    fn perform(self) {
        match Msg {
            Msg::Hello => println!("Hello!"),
            Msg::World => println!("Hello!"),
        }
    }
}
let mut pool3 = Pool::<Deque<Msg>>::new(deque::Options::default());
pool3.push(Msg::Hello);
pool3.push(Msg::World);

// scoped, dynamic dispatch
let mut items = [0usize; 8];
crossbeam::scope(|scope| {
    let mut pool = Pool::<Deque<Task>>::scoped(&scope, deque::Options::default());
    for (i, e) in items.iter_mut().enumerate() {
        pool.push(move || *e = i)
    }
});

// scoped, static dispatch
struct Work<'a> { i: usize, e: &'a mut usize }
impl Job for Work {
    fn perform(self) {
        *self.e = i;
    }
}
crossbeam::scope(|scope| {
    let mut pool = Pool::<Deque<Work>>::scoped(&scope, deque::Options::default());
    for (i, e) in items.iter_mut().enumerate() {
        pool.push(Work { i: i, e: e });
    }
}
```

## Design

I was inspired to start this after attending the [January 2016][jan2016_meetup]
Bay Area Rust meetup and watching Aaron Turon's presentation about `crossbeam`.
The initial version of this crate supported only boxed, dynamically-dispatched
tasks (closures). At this stage, it was essentially `scoped_threadpool` with a
different task-passing mechanism.

The first breakthrough came when I realized that a simple `Job` trait would make
both static and dynamic dispatch possible:

```rust
pub trait Job: Send {
    fn perform(self);
}

pub struct Task<'a>(Box<FnBox + Send + 'a>);

impl<'a> Job for Task<'a> {
    #[inline]
    fn perform(self) {
        let Task(task) = self;
        task.call_box();
    }
}

impl<'a, F> From<F> for Task<'a> where F: FnOnce() + Send + 'a
{
    #[inline]
    fn from(f: F) -> Task<'a> {
        Task(Box::new(f))
    }
}
```

Much better: static dispatch with no additional allocation, while maintaining
convenient dynamic dispatch. I soon added support for `'static` jobs that run
outside a `crossbeam::scope`.

At this point, I was pretty happy with the result. However, `scoped_threadpool`,
with its simple `Arc<Mutex<Receiver>>` method for delivering tasks to worker
threads, stayed in the back of my mind. Eventually I decided to write benchmarks
comparing the two, but since it didn't support static dispatch, I wrote a
parallel set of pool and worker types. That's when the next insight came: the
pool implementations were highly similar, except for some book-keeping related
to how each implemented job communication. So I factored out those details into
a family of traits:

```rust
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
```

The implementation of `Pool` became rather straightforward:

```rust
pub struct Pool<C>
    where C: Crew
{
    crew: C,
}

impl<'scope, C, J> Pool<C>
    where J: Job + 'scope,
          C: Crew<Job = J> + 'scope
{
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

    pub fn push<F>(&mut self, f: F)
        where J: From<F>
    {
        self.crew.give(f);
    }
}
```

The icing on the cake: this crate never uses `unsafe`:

    $ grep -r --include=*.rs unsafe .
    $

[As dbaupp pointed out][dbaupp], however, the current design has a major
drawback compared to other scoped threadpools because of this, since it cannot
reuse the same pool for different scopes throughout the lifetime of an
application.

[jan2016_meetup]: https://air.mozilla.org/bay-area-rust-meetup-january-2016/?a
[dbaupp]: https://www.reddit.com/r/rust/comments/43ajja/kirk_a_highlyflexible_thread_pool/czh0nmi

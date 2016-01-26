#![cfg_attr(feature = "nightly",
            feature(recover))]

#[macro_use]
extern crate log;

extern crate crossbeam;
extern crate env_logger;
extern crate kirk;

#[test]
fn it_works() {
    use std::iter;

    const ITEMS: usize = 100_000;
    let mut results = iter::repeat((0, 0)).take(ITEMS).collect::<Vec<_>>();

    let options = kirk::Options::default();
    crossbeam::scope(|scope| {
        let mut pool = kirk::Pool::<kirk::Task>::scoped(&scope, options);
        for (i, result) in results.iter_mut().enumerate() {
            pool.push(move || {
                *result = (i, i + 1);
            });
        }
    });

    for (i, j) in results {
        assert_eq!(j, i + 1);
    }
}

#[test]
#[cfg(feature = "nightly")]
fn it_doesnt_bail() {
    let options = kirk::Options::default();
    crossbeam::scope(|scope| {
        let mut pool = kirk::Pool::<kirk::Task>::scoped(&scope, options);
        pool.push(|| panic!(""));
    });
}

static STUFF: [u8; 128] = [0; 128];

struct IncrementJob {
    item: &'static u8,
    tx: std::sync::mpsc::Sender<u8>,
}

impl kirk::Job for IncrementJob {
    type Product = ();

    fn perform(self) {
        self.tx.send(*self.item + 1).unwrap();
    }
}

#[cfg(feature = "nightly")]
impl std::panic::RecoverSafe for IncrementJob { }

#[test]
fn static_works() {
    use std::thread;

    let collector = {
        let options = kirk::Options::default();
        let mut pool = kirk::Pool::<IncrementJob>::new(options);
        let rx = {
            let (tx, rx) = std::sync::mpsc::channel();
            for x in STUFF.iter() {
                pool.push(IncrementJob { item: x, tx: tx.clone() });
            }
            rx
        };
        thread::spawn(move || {
            for y in rx.iter() {
                assert_eq!(y, 1);
            }
        })
    };
    collector.join().unwrap();
}

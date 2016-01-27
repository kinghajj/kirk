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

    let options = kirk::crew::deque::Options::default();
    crossbeam::scope(|scope| {
        let mut pool = kirk::Pool::<kirk::Deque<kirk::Task>>::scoped(&scope, options);
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
    let options = kirk::crew::deque::Options::default();
    crossbeam::scope(|scope| {
        let mut pool = kirk::Pool::<kirk::Deque<kirk::Task>>::scoped(&scope, options);
        pool.push(|| panic!(""));
    });
}

static STUFF: [u8; 128] = [0; 128];

#[test]
fn static_works() {
    use std::thread;

    let collector = {
        let options = kirk::crew::deque::Options::default();
        let mut pool = kirk::Pool::<kirk::Deque<kirk::Task>>::new(options);
        let rx = {
            let (tx, rx) = std::sync::mpsc::channel();
            for x in STUFF.iter() {
                let tx = tx.clone();
                pool.push(move || {
                    tx.send(*x + 1).unwrap();
                    drop(tx);
                })
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

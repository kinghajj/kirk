extern crate crossbeam;
extern crate env_logger;
extern crate kirk;

#[test]
fn it_works() {
    use std::iter;

    env_logger::init().unwrap();

    const ITEMS: usize = 100_000;
    let mut results = iter::repeat((0, 0)).take(ITEMS).collect::<Vec<_>>();

    let options = kirk::Options::default();
    crossbeam::scope(|scope| {
        let mut pool = kirk::Pool::<kirk::Task>::new(&scope, options);
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
        let mut pool = kirk::Pool::<kirk::Task>::new(&scope, options);
        pool.push(|| panic!(""));
    });
}

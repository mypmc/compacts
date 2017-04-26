#![feature(test)]

#[macro_use]
extern crate thunk;
extern crate test;

use self::test::Bencher;

#[bench]
fn bench_eval(bench: &mut Bencher) {
    let mut expr = eval!(0);
    bench.iter(|| expr = eval!(0));
}

#[bench]
fn bench_lazy(bench: &mut Bencher) {
    let mut expr = lazy!(0);
    bench.iter(|| expr = lazy!(0));
}

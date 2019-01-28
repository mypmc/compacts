#![feature(test)]

extern crate compacts;
extern crate rand;
extern crate test;

use compacts::bits::*;
use rand::Rng;
use test::Bencher;

use rand::prelude::*;

fn rng() -> ThreadRng {
    rand::thread_rng()
}

#[bench]
fn rrr15_encode(bench: &mut Bencher) {
    bench.iter(|| rrr15::encode(rng().gen()));
}
#[bench]
fn rrr15_decode(bench: &mut Bencher) {
    let (o, c) = rrr15::encode(rng().gen());
    bench.iter(|| rrr15::decode(o, c));
}

#[bench]
fn rrr31_encode(bench: &mut Bencher) {
    bench.iter(|| rrr31::encode(rng().gen()));
}
#[bench]
fn rrr31_decode(bench: &mut Bencher) {
    let (o, c) = rrr31::encode(rng().gen());
    bench.iter(|| rrr31::decode(o, c));
}

#[bench]
fn rrr63_encode(bench: &mut Bencher) {
    bench.iter(|| rrr63::encode(rng().gen()));
}
#[bench]
fn rrr63_decode(bench: &mut Bencher) {
    let (o, c) = rrr63::encode(rng().gen());
    bench.iter(|| rrr63::decode(o, c));
}

// #[bench]
// fn rrr127_encode(bench: &mut Bencher) {
//     let h = rng().gen::<u64>() as u128;
//     let l = rng().gen::<u64>() as u128;
//     bench.iter(|| rrr127::encode(h << 64 | l));
// }
// #[bench]
// fn rrr127_decode(bench: &mut Bencher) {
//     let h = rng().gen::<u64>() as u128;
//     let l = rng().gen::<u64>() as u128;
//     let (o, c) = rrr127::encode(h << 64 | l);
//     bench.iter(|| rrr127::decode(o, c));
// }

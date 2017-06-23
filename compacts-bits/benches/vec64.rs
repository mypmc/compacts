#![feature(test)]

extern crate compacts_bits;
extern crate rand;
extern crate test;

use test::Bencher;
use rand::Rng;

use compacts_bits::*;
// use compacts_bits::ops::*;

fn random_insert<R>(vec: &mut Vec64, rng: &mut R, size: u64, max: u64)
where
    R: Rng,
{
    for _ in 0..rng.gen_range(0, size) {
        vec.insert(rng.gen_range(0, max));
    }
}

const SIZE: u64 = 1 << 16;
const MAX1: u64 = 1 << 20;
const MAX2: u64 = 1 << 40;

#[bench]
fn optimize(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let mut vec = Vec64::new();
    random_insert(&mut vec, &mut rng, SIZE, MAX2);
    bench.iter(|| vec.optimize());
}

#[bench]
fn index(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let mut vec = Vec64::new();
    random_insert(&mut vec, &mut rng, SIZE, MAX2);
    bench.iter(|| vec[rng.gen()]);
}

#[bench]
fn contains(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let mut vec = Vec64::new();
    random_insert(&mut vec, &mut rng, SIZE, MAX2);
    bench.iter(|| vec.contains(rng.gen()));
}

#[bench]
fn insert(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let mut vec = Vec64::new();
    random_insert(&mut vec, &mut rng, SIZE, MAX2);
    bench.iter(|| vec.insert(rng.gen_range(0, MAX1)));
}

#[bench]
fn remove(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let mut vec = Vec64::new();
    random_insert(&mut vec, &mut rng, SIZE, MAX2);
    bench.iter(|| vec.remove(rng.gen()));
}

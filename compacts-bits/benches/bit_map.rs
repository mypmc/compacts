#![feature(test)]

extern crate compacts_bits;
extern crate rand;
extern crate test;

use test::Bencher;
use rand::Rng;

use compacts_bits::*;
// use compacts_bits::ops::*;

fn random_insert<R>(map: &mut BitMap, rng: &mut R, size: u64, max: u64)
where
    R: Rng,
{
    for _ in 0..rng.gen_range(0, size) {
        map.insert(rng.gen_range(0, max));
    }
}

const SIZE: u64 = 1 << 16;
const MAX1: u64 = 1 << 20;
const MAX2: u64 = 1 << 40;

#[bench]
fn index(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let mut v1 = BitMap::new();
    random_insert(&mut v1, &mut rng, SIZE, MAX2);
    bench.iter(|| v1[rng.gen()]);
}

#[bench]
fn contains(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let mut v1 = BitMap::new();
    random_insert(&mut v1, &mut rng, SIZE, MAX2);
    bench.iter(|| v1.contains(rng.gen()));
}

#[bench]
fn insert(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let mut v1 = BitMap::new();
    random_insert(&mut v1, &mut rng, SIZE, MAX2);
    bench.iter(|| v1.insert(rng.gen_range(0, MAX1)));
}

#[bench]
fn remove(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let mut v1 = BitMap::new();
    random_insert(&mut v1, &mut rng, SIZE, MAX2);
    bench.iter(|| v1.remove(rng.gen()));
}

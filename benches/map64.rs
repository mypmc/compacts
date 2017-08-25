#![feature(test)]

extern crate compacts;
extern crate rand;
extern crate test;

use test::Bencher;
use rand::Rng;
use compacts::bits::*;
use compacts::dict::*;

fn random_insert<R>(vec: &mut Map64, rng: &mut R, size: u64, max: u64)
where
    R: Rng,
{
    for _ in 0..size {
        vec.insert(rng.gen_range(0, max));
    }
    vec.optimize();
}

macro_rules! genvec {
    ( $vec:ident, $rng:expr ) => {
        genvec!($vec, $rng, SIZE, MAX2);
    };
    ( $vec:ident, $rng:expr, $size:expr, $maxn:expr ) => {
        {
            let mut $vec = Map64::new();
            random_insert(&mut $vec, &mut $rng, $size, $maxn);
            $vec
        }
    }
}

const SIZE: u64 = 1 << 16;
const MAX1: u64 = 1 << 16;
const MAX2: u64 = 1 << 32;

#[bench]
fn optimize(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let mut vec = genvec!(vec, rng);
    bench.iter(|| vec.optimize());
}

#[bench]
fn index(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let vec = genvec!(vec, rng);
    bench.iter(|| vec[rng.gen()]);
}

#[bench]
fn contains(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let vec = genvec!(vec, rng);
    bench.iter(|| vec.contains(rng.gen()));
}

#[bench]
fn insert(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let mut vec = genvec!(vec, rng);
    bench.iter(|| vec.insert(rng.gen_range(0, MAX1)));
}

#[bench]
fn remove(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let mut vec = genvec!(vec, rng);
    bench.iter(|| vec.remove(rng.gen()));
}

#[bench]
fn pop_count(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let vec = genvec!(vec, rng);
    bench.iter(|| vec.count1());
}

#[bench]
fn intersection(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let mut vec1 = genvec!(vec, rng);
    let vec2 = genvec!(vec, rng);
    bench.iter(|| vec1.intersection_with(&vec2));
}

#[bench]
fn union(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let mut vec1 = genvec!(vec, rng);
    let vec2 = genvec!(vec, rng);
    bench.iter(|| vec1.union_with(&vec2));
}

#[bench]
fn difference(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let mut vec1 = genvec!(vec, rng);
    let vec2 = genvec!(vec, rng);
    bench.iter(|| vec1.difference_with(&vec2));
}

#[bench]
fn symmetric_difference(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let mut vec1 = genvec!(vec, rng);
    let vec2 = genvec!(vec, rng);
    bench.iter(|| vec1.symmetric_difference_with(&vec2));
}

#[bench]
fn rank(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let vec = genvec!(vec, rng);
    bench.iter(|| vec.rank1(rng.gen()));
}

#[bench]
fn select(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let vec = genvec!(vec, rng);
    bench.iter(|| vec.select1(rng.gen()));
}

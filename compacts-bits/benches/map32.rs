#![feature(test)]

extern crate compacts_bits;
extern crate rand;
extern crate test;

use test::Bencher;
use rand::Rng;

use compacts_bits::*;
use self::pair::*;

macro_rules! bit_vec {
    ( 0, 1, $rng:expr ) => {{ Map32::new() }};
    ( $size:expr, $end:expr, $rng:expr ) => {{
        bit_vec!($size, 0, $end, $rng)
    }};
    ( $size:expr, $start:expr, $end:expr, $rng:expr ) => {{
        let mut vec = Map32::new();
        if $size > 1 {
            for _ in 0..$size {
                let gen = $rng.gen_range($start, $end);
                vec.insert(gen);
            }
        }
        vec.optimize();
        vec
    }};
}

#[bench]
fn contains(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let v1 = bit_vec!(65_000, 2_000_000, rng);
    bench.iter(|| v1.contains(rng.gen()));
}

#[bench]
fn insert(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let mut v1 = bit_vec!(65_000, 2_000_000, rng);
    bench.iter(|| v1.insert(rng.gen()));
}

#[bench]
fn remove(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let mut v1 = bit_vec!(65_000, 2_000_000, rng);
    bench.iter(|| v1.remove(rng.gen()));
}

#[bench]
fn clone(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let mut v1 = bit_vec!(65_000, 2_000_000, rng);
    bench.iter(|| v1 = v1.clone());
}

#[bench]
fn collect(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let v1 = bit_vec!(65_000, 2_000_000, rng);
    bench.iter(|| test::black_box(v1.iter().collect::<Vec<u32>>()));
}

const SIZE: usize = 65_000;
const RANGE1: u32 = 1_000_000;
const RANGE2: u32 = 100_000_000;

#[bench]
fn intersection(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let mut v1 = bit_vec!(SIZE, RANGE1, rng);
    let v2 = &(bit_vec!(SIZE, RANGE2, rng));
    bench.iter(|| v1.intersection_with(v2));
}

#[bench]
fn union(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let mut v1 = bit_vec!(SIZE, RANGE1, rng);
    let v2 = &(bit_vec!(SIZE, RANGE2, rng));
    bench.iter(|| v1.union_with(v2));
}

#[bench]
fn union_lazy(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let v1 = bit_vec!(SIZE, RANGE1, rng);
    let v2 = &(bit_vec!(SIZE, RANGE2, rng));
    let bv = &(bit_vec!(0, 1, rng));
    bench.iter(|| {
        let mut v = v1.clone();
        v.union_with(v2);
        v.intersection_with(bv);
    });
}

#[bench]
fn difference(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let mut v1 = bit_vec!(SIZE, RANGE1, rng);
    let v2 = &(bit_vec!(SIZE, RANGE2, rng));
    bench.iter(|| v1.difference_with(v2));
}

#[bench]
fn difference_lazy(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let v1 = bit_vec!(SIZE, RANGE1, rng);
    let v2 = &(bit_vec!(SIZE, RANGE2, rng));
    let bv = &(bit_vec!(0, 1, rng));
    bench.iter(|| {
        let mut v = v1.clone();
        v.difference_with(v2);
        v.intersection_with(bv);
    });
}

#[bench]
fn symmetric_difference(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let mut v1 = bit_vec!(SIZE, RANGE1, rng);
    let v2 = &(bit_vec!(SIZE, RANGE2, rng));
    bench.iter(|| v1.symmetric_difference_with(v2));
}

#[bench]
fn symmetric_difference_lazy(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let v1 = bit_vec!(SIZE, RANGE1, rng);
    let v2 = &(bit_vec!(SIZE, RANGE2, rng));
    let bv = &(bit_vec!(0, 1, rng));
    bench.iter(|| {
        let mut v = v1.clone();
        v.symmetric_difference_with(v2);
        v.intersection_with(bv);
    });
}
#[bench]
fn small_rank(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let v1 = bit_vec!(SIZE, RANGE1, rng);
    let i = rng.gen();
    bench.iter(|| v1.rank1(i));
}
#[bench]
fn large_rank(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let v1 = bit_vec!(SIZE, RANGE2, rng);
    let i = rng.gen();
    bench.iter(|| v1.rank1(i));
}

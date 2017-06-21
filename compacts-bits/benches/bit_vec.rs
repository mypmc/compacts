#![feature(test)]

extern crate compacts_bits;
extern crate rand;
extern crate test;

use test::Bencher;
use rand::Rng;

use compacts_bits::*;
use compacts_bits::ops::*;

macro_rules! bit_vec {
    ( 0, 1, $rng:expr ) => {{ BitVec::new() }};
    ( $size:expr, $end:expr, $rng:expr ) => {{
        bit_vec!($size, 0, $end, $rng)
    }};
    ( $size:expr, $start:expr, $end:expr, $rng:expr ) => {{
        let mut vec = BitVec::new();
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
fn bit_vec_clone(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let mut v1 = bit_vec!(65_000, 2000000, rng);
    bench.iter(|| v1 = v1.clone());
}

#[bench]
fn bit_vec_collect(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let v1 = bit_vec!(65_000, 2000000, rng);
    bench.iter(|| test::black_box(v1.iter().collect::<Vec<u32>>()));
}

const SIZE: usize = 65000;
const RANGE1: u32 = 1500000;
const RANGE2: u32 = 100000000;

// Commenting out `intersection_with(bv)` line causes evaluation of thunked computations.
// But in testing (`../tests`), this doesn't happen.
// My guess: because of `cargo bench`? I have no confidence.

#[bench]
fn bit_vec_intersection(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let mut v1 = bit_vec!(SIZE, RANGE1, rng);
    let v2 = &(bit_vec!(SIZE, RANGE2, rng));
    bench.iter(|| v1.intersection_with(v2));
}

#[bench]
fn bit_vec_union(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let mut v1 = bit_vec!(SIZE, RANGE1, rng);
    let v2 = &(bit_vec!(SIZE, RANGE2, rng));
    let bv = &(bit_vec!(0, 1, rng));
    bench.iter(|| {
        v1.union_with(v2);
        v1.intersection_with(bv);
    });
}

#[bench]
fn bit_vec_difference(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let mut v1 = bit_vec!(SIZE, RANGE1, rng);
    let v2 = &(bit_vec!(SIZE, RANGE2, rng));
    let bv = &(bit_vec!(0, 1, rng));
    bench.iter(|| {
        v1.union_with(v2);
        v1.intersection_with(bv);
    });
}

#[bench]
fn bit_vec_symmetric_difference(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let mut v1 = bit_vec!(SIZE, RANGE1, rng);
    let v2 = &(bit_vec!(SIZE, RANGE2, rng));
    let bv = &(bit_vec!(0, 1, rng));
    bench.iter(|| {
        v1.symmetric_difference_with(v2);
        v1.intersection_with(bv);
    });
}

#[bench]
fn small_bit_vec_rank(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let v1 = bit_vec!(SIZE, RANGE1, rng);
    let i = rng.gen();
    bench.iter(|| v1.rank1(i));
}
#[bench]
fn large_bit_vec_rank(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let v1 = bit_vec!(SIZE, RANGE2, rng);
    let i = rng.gen();
    bench.iter(|| v1.rank1(i));
}

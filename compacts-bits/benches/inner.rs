#![feature(test)]
#![feature(inclusive_range)]

// #[macro_use]
// extern crate log;
// extern crate env_logger;
extern crate rand;
extern crate test;
extern crate compacts_bits;

use rand::Rng;
use test::Bencher;
use compacts_bits::internal::*;
use compacts_bits::ops::*;

use std::ops::RangeInclusive;
use std::mem;

const SIZE: u16 = 8000;
const END: u16 = 65535;
const BIAS1: u16 = 87;
const BIAS2: u16 = 38;


macro_rules! random {
    ( $repr:expr, $size:expr, $end:expr, $rng:expr ) => {{
        for _ in 0..$size {
            let gen = $rng.gen_range(0, $end);
            $repr.insert(gen);
        }
        $repr
    }};
}
macro_rules! biased {
    ( $repr:expr, $size:expr, $block:expr ) => {{
        // let step = if $block > 1024 { 1024 } else { $block };
        for k in 1..($block+1) {
            for i in ($size + k*k)..($size + (k*k) + 3) {
                $repr.insert(i);
            }
        }
        $repr
    }};
    ( $repr:expr, $size:expr ) => {{
        biased!($repr, $size, 500)
    }};
}

macro_rules! init_random_seq16 {
    ( $seq:ident ) => {
        let mut rng = rand::thread_rng();
        let mut $seq = Seq16::new();
        random!(&mut $seq, SIZE, END, rng);
    };
}
macro_rules! init_random_seq64 {
    ( $seq:ident ) => {
        let mut rng = rand::thread_rng();
        let mut $seq = Seq64::new();
        random!(&mut $seq, SIZE, END, rng);
    };
}

macro_rules! init_biased_seq64 {
    ( $seq:ident, $bias:expr, $block:expr ) => {
        let mut $seq = Seq64::new();
        biased!(&mut $seq, $bias, $block);
    };
    ( $seq:ident, $bias:expr ) => {
        let mut $seq = Seq64::new();
        biased!(&mut $seq, $bias);
    };
}

#[bench]
fn random_vec16_as_rle16(bench: &mut Bencher) {
    init_random_seq16!(seq);
    let seq = &seq;

    let size = mem::size_of::<u16>();
    let len = seq.vector.len();
    let w = seq.weight;
    print!("seq16({:6?}byte l:{:?} w:{:?} ) ", size * len, len, w);

    let rle = Rle16::from(seq);
    let size = mem::size_of::<RangeInclusive<u16>>(); //4
    let len = rle.ranges.len();
    let w = rle.weight;
    print!("rle16({:6?}byte l:{:?} w:{:?} ) ", size * len, len, w);

    bench.iter(|| Rle16::from(seq));
}

#[bench]
fn random_vec64_as_rle16(bench: &mut Bencher) {
    init_random_seq64!(seq);
    let seq = &seq;

    let size = mem::size_of::<u64>();
    let len = seq.vector.len();
    let w = seq.weight;
    print!("seq64({:6?}byte l:{:?} w:{:?} ) ", size * len, len, w);

    let rle = Rle16::from(seq);
    let size = mem::size_of::<RangeInclusive<u16>>(); //4
    let len = rle.ranges.len();
    let w = rle.weight;
    print!("rle16({:6?}byte l:{:?} w:{:?} ) ", size * len, len, w);

    bench.iter(|| Rle16::from(seq));
}

#[bench]
fn random_rle16_as_vec16(bench: &mut Bencher) {
    init_random_seq16!(seq);
    let rle = &Rle16::from(seq);
    bench.iter(|| Seq16::from(rle))
}

#[bench]
fn random_rle16_as_vec64(bench: &mut Bencher) {
    init_random_seq64!(seq);
    let rle = &Rle16::from(seq);
    bench.iter(|| Seq64::from(rle))
}

#[bench]
fn random_seq16_intersection_seq16(bench: &mut Bencher) {
    init_random_seq16!(seq1);
    init_random_seq16!(seq2);
    bench.iter(|| seq1.intersection_with(&seq2))
}

#[bench]
fn random_seq64_intersection_seq64(bench: &mut Bencher) {
    init_random_seq64!(seq1);
    init_random_seq64!(seq2);
    bench.iter(|| seq1.intersection_with(&seq2))
}

#[bench]
fn random_rle16_intersection_rle16(bench: &mut Bencher) {
    init_random_seq64!(seq1);
    init_random_seq64!(seq2);
    let rle1 = &Rle16::from(seq1);
    let rle2 = &Rle16::from(seq2);
    bench.iter(|| rle1.intersection(&rle2))
}

#[bench]
fn biased_rle16_intersection_rle16(bench: &mut Bencher) {
    init_biased_seq64!(seq1, BIAS1);
    init_biased_seq64!(seq2, BIAS2);
    let rle1 = &Rle16::from(seq1);
    let rle2 = &Rle16::from(seq2);
    bench.iter(|| rle1.intersection(&rle2))
}

#[bench]
fn biased_rle16_union_rle16(bench: &mut Bencher) {
    init_biased_seq64!(seq1, BIAS1);
    init_biased_seq64!(seq2, BIAS2);
    let rle1 = &Rle16::from(seq1);
    let rle2 = &Rle16::from(seq2);
    bench.iter(|| rle1.union(&rle2))
}

#[bench]
fn biased_rle16_difference_rle16(bench: &mut Bencher) {
    init_biased_seq64!(seq1, BIAS1);
    init_biased_seq64!(seq2, BIAS2);
    let rle1 = &Rle16::from(seq1);
    let rle2 = &Rle16::from(seq2);
    bench.iter(|| rle1.difference(&rle2))
}

#[bench]
fn biased_rle16_symmetric_difference_rle16(bench: &mut Bencher) {
    init_biased_seq64!(seq1, BIAS1);
    init_biased_seq64!(seq2, BIAS2);
    let rle1 = &Rle16::from(seq1);
    let rle2 = &Rle16::from(seq2);
    bench.iter(|| rle1.symmetric_difference(&rle2))
}

#[bench]
fn rle16_ranges_5(bench: &mut Bencher) {
    init_biased_seq64!(seq1, BIAS1, 5);
    init_biased_seq64!(seq2, BIAS2, 5);
    let rle1 = &Rle16::from(seq1);
    let rle2 = &Rle16::from(seq2);

    let size = mem::size_of::<RangeInclusive<u16>>(); //4
    let len = rle1.ranges.len();
    let w = rle1.weight;
    print!("{:6?}byte len:{:5?} weight:{:5?} ", size * len, len, w);

    bench.iter(|| rle1.union(&rle2))
}

#[bench]
fn rle16_ranges_10(bench: &mut Bencher) {
    init_biased_seq64!(seq1, BIAS1, 10);
    init_biased_seq64!(seq2, BIAS2, 10);
    let rle1 = &Rle16::from(seq1);
    let rle2 = &Rle16::from(seq2);

    let size = mem::size_of::<RangeInclusive<u16>>(); //4
    let len = rle1.ranges.len();
    let w = rle1.weight;
    print!("{:6?}byte len:{:5?} weight:{:5?} ", size * len, len, w);

    bench.iter(|| rle1.union(&rle2))
}

#[bench]
fn rle16_ranges_100(bench: &mut Bencher) {
    init_biased_seq64!(seq1, BIAS1, 100);
    init_biased_seq64!(seq2, BIAS2, 100);
    let rle1 = &Rle16::from(seq1);
    let rle2 = &Rle16::from(seq2);

    let size = mem::size_of::<RangeInclusive<u16>>(); //4
    let len = rle1.ranges.len();
    let w = rle1.weight;
    print!("{:6?}byte len:{:5?} weight:{:5?} ", size * len, len, w);

    bench.iter(|| rle1.union(&rle2))
}

#[bench]
fn rle16_ranges_500(bench: &mut Bencher) {
    init_biased_seq64!(seq1, BIAS1, 500);
    init_biased_seq64!(seq2, BIAS2, 500);
    let rle1 = &Rle16::from(seq1);
    let rle2 = &Rle16::from(seq2);

    let size = mem::size_of::<RangeInclusive<u16>>(); //4
    let len = rle1.ranges.len();
    let w = rle1.weight;
    print!("{:6?}byte len:{:5?} weight:{:5?} ", size * len, len, w);

    bench.iter(|| rle1.union(&rle2))
}

#[bench]
fn rle16_ranges_1024(bench: &mut Bencher) {
    init_biased_seq64!(seq1, BIAS1, 1024);
    init_biased_seq64!(seq2, BIAS2, 1024);
    let rle1 = &Rle16::from(seq1);
    let rle2 = &Rle16::from(seq2);

    let size = mem::size_of::<RangeInclusive<u16>>(); //4
    let len = rle1.ranges.len();
    let w = rle1.weight;
    print!("{:6?}byte len:{:5?} weight:{:5?} ", size * len, len, w);

    bench.iter(|| rle1.union(&rle2))
}

#[bench]
fn rle16_ranges_1500(bench: &mut Bencher) {
    init_biased_seq64!(seq1, BIAS1, 1500);
    init_biased_seq64!(seq2, BIAS2, 1500);
    let rle1 = &Rle16::from(seq1);
    let rle2 = &Rle16::from(seq2);

    let size = mem::size_of::<RangeInclusive<u16>>(); //4
    let len = rle1.ranges.len();
    let w = rle1.weight;
    print!("{:6?}byte len:{:5?} weight:{:5?} ", size * len, len, w);

    bench.iter(|| rle1.union(&rle2))
}

#[bench]
fn rle16_ranges_2000(bench: &mut Bencher) {
    init_biased_seq64!(seq1, BIAS1, 2000);
    init_biased_seq64!(seq2, BIAS2, 2000);
    let rle1 = &Rle16::from(seq1);
    let rle2 = &Rle16::from(seq2);

    let size = mem::size_of::<RangeInclusive<u16>>(); //4
    let len = rle1.ranges.len();
    let w = rle1.weight;
    print!("{:6?}byte len:{:5?} weight:{:5?} ", size * len, len, w);

    bench.iter(|| rle1.union(&rle2))
}

#[bench]
fn rle16_ranges_3000(bench: &mut Bencher) {
    init_biased_seq64!(seq1, BIAS1, 3000);
    init_biased_seq64!(seq2, BIAS2, 3000);
    let rle1 = &Rle16::from(seq1);
    let rle2 = &Rle16::from(seq2);

    let size = mem::size_of::<RangeInclusive<u16>>(); //4
    let len = rle1.ranges.len();
    let w = rle1.weight;
    print!("{:6?}byte len:{:5?} weight:{:5?} ", size * len, len, w);

    bench.iter(|| rle1.union(&rle2))
}

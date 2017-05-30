#![feature(test)]
#![feature(step_by)]

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

const SIZE: u16 = 5000;
const END: u16 = 5000;

const BIAS1: u16 = 172;
const BIAS2: u16 = 171;

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
    ( $repr:expr, $size:expr, $block:expr, $rng:expr ) => {{
        for k in (0..65535).step_by($size/2).take($block) {
            let gen = $rng.gen_range(k, ::std::u16::MAX);
            for i in gen..gen+10 {
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
        let mut rng = rand::thread_rng();
        let mut $seq = Seq64::new();
        biased!(&mut $seq, $bias, $block, rng);
    };
    ( $seq:ident, $bias:expr ) => {
        init_biased_seq64!($seq, $bias, 128);
    };
}

#[bench]
fn random_vec16_mem_in_rle16(bench: &mut Bencher) {
    init_random_seq16!(seq);
    let seq = &seq;
    bench.iter(|| seq.mem_in_rle());
}
#[bench]
fn random_vec64_mem_in_rle16(bench: &mut Bencher) {
    init_random_seq64!(seq);
    let seq = &seq;
    bench.iter(|| seq.mem_in_rle());
}

#[bench]
fn biased_vec16_mem_in_rle16(bench: &mut Bencher) {
    init_biased_seq64!(seq, BIAS1, 2000);
    let seq = &Seq16::from(seq);
    bench.iter(|| seq.mem_in_rle());
}
#[bench]
fn biased_vec64_mem_in_rle16(bench: &mut Bencher) {
    init_biased_seq64!(seq, BIAS1, 2000);
    let seq = &seq;
    bench.iter(|| seq.mem_in_rle());
}

#[bench]
fn random_vec16_as_rle16(bench: &mut Bencher) {
    init_random_seq16!(seq);
    let seq = &seq;
    bench.iter(|| Rle16::from(seq));
}

#[bench]
fn random_vec64_as_rle16(bench: &mut Bencher) {
    init_random_seq64!(seq);
    let seq = &seq;
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

macro_rules! rle16_biased_of {
    ( $bench:expr, ( $b1:expr, $s1:expr ) , ( $b2:expr, $s2:expr ) ) => {
        init_biased_seq64!(seq1, $b1, $s1);
        let v1 = Rle16::from(seq1.clone());
        let v2 = Seq16::from(seq1.clone());
        let v3 = Seq64::from(seq1.clone());
        print!("{:?} {:?} {:?} ", v1, v2, v3);
        $bench.iter(|| Rle16::from(&v3))
    }
}
macro_rules! rle16_random_of {
    ( $bench:expr ) => {
        init_random_seq64!(seq1);
        let v1 = Rle16::from(seq1.clone());
        let v2 = Seq16::from(seq1.clone());
        let v3 = Seq64::from(seq1.clone());
        print!("{:?} {:?} {:?} ", v1, v2, v3);
        $bench.iter(|| Rle16::from(&v3))
    }
}

#[bench]
fn rle16_random_of(bench: &mut Bencher) {
    rle16_random_of!(bench);
}
#[bench]
fn rle16_ranges_64(bench: &mut Bencher) {
    rle16_biased_of!(bench, (BIAS1, 64), (BIAS2, 64));
}
#[bench]
fn rle16_ranges_128(bench: &mut Bencher) {
    rle16_biased_of!(bench, (BIAS1, 128), (BIAS2, 128));
}
#[bench]
fn rle16_ranges_256(bench: &mut Bencher) {
    rle16_biased_of!(bench, (BIAS1, 256), (BIAS2, 256));
}
#[bench]
fn rle16_ranges_512(bench: &mut Bencher) {
    rle16_biased_of!(bench, (BIAS1, 512), (BIAS2, 512));
}
#[bench]
fn rle16_ranges_1024(bench: &mut Bencher) {
    rle16_biased_of!(bench, (BIAS1, 1024), (BIAS2, 1024));
}
#[bench]
fn rle16_ranges_2048(bench: &mut Bencher) {
    rle16_biased_of!(bench, (BIAS1, 2048), (BIAS2, 2048));
}
#[bench]
fn rle16_ranges_4096(bench: &mut Bencher) {
    rle16_biased_of!(bench, (BIAS1, 4096), (BIAS2, 4096));
}
#[bench]
fn rle16_ranges_8192(bench: &mut Bencher) {
    rle16_biased_of!(bench, (BIAS1, 8192), (BIAS2, 8192));
}

#![feature(test)]

extern crate compacts;
extern crate rand;
extern crate snap;
extern crate test;
extern crate zstd;

use std::{fs, io};
use test::Bencher;
use rand::Rng;
use compacts::bits::Set as BitSet;
use compacts::bits::*;

macro_rules! gen_bitset {
    ( 0, 1, $rng:expr ) => {
        {
            BitSet::new()
        }
    };
    ( $size:expr, $end:expr, $rng:expr ) => {
        {
            gen_bitset!($size, 0, $end, $rng)
        }
    };
    ( $size:expr, $start:expr, $end:expr, $rng:expr ) => {
        {
            let mut bits = BitSet::new();
            if $size > 1 {
                for _ in 0..$size {
                    let gen = $rng.gen_range($start, $end);
                    bits.put(gen, true);
                }
                bits.optimize();
            }
            bits
        }
    };
}

const SIZE: usize = 65_000;
const RANGE1: u32 = 150_000;
const RANGE2: u32 = 100_000_000;

#[bench]
fn bitset_get(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let v1 = gen_bitset!(SIZE, RANGE1, rng);
    bench.iter(|| v1.get(rng.gen()));
}

#[bench]
fn bitset_put_true(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let mut v1 = gen_bitset!(SIZE, RANGE1, rng);
    bench.iter(|| v1.put(rng.gen(), true));
}

#[bench]
fn bitset_put_false(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let mut v1 = gen_bitset!(SIZE, RANGE1, rng);
    bench.iter(|| v1.put(rng.gen(), false));
}

#[bench]
fn bitset_bits_65000_150000(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let v1 = gen_bitset!(SIZE, RANGE1, rng);
    bench.iter(|| v1.bits().collect::<Vec<u32>>());
}
#[bench]
fn bitset_bits_65000_100000000(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let v1 = gen_bitset!(SIZE, RANGE2, rng);
    bench.iter(|| v1.bits().collect::<Vec<u32>>());
}
#[bench]
fn bitset_bits_1000000_100000000(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let v1 = gen_bitset!(1_000_000, RANGE2, rng);
    bench.iter(|| v1.bits().collect::<Vec<u32>>());
}
#[bench]
fn bitset_bits_1000000_u32max(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let v1 = gen_bitset!(1_000_000, !0, rng);
    bench.iter(|| v1.bits().collect::<Vec<u32>>());
}

#[bench]
fn bitset_clone_small(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let mut v1 = gen_bitset!(SIZE, RANGE1, rng);
    bench.iter(|| v1 = v1.clone());
}
#[bench]
fn bitset_clone_large(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let mut v1 = gen_bitset!(SIZE, RANGE2, rng);
    bench.iter(|| v1 = v1.clone());
}

#[bench]
fn bitset_and(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let v1 = gen_bitset!(SIZE, RANGE1, rng);
    let v2 = gen_bitset!(SIZE, RANGE2, rng);
    bench.iter(|| v1.and(&v2));
}

#[bench]
fn bitset_or(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let v1 = gen_bitset!(SIZE, RANGE1, rng);
    let v2 = gen_bitset!(SIZE, RANGE2, rng);
    let bv = gen_bitset!(0, 1, rng);
    bench.iter(|| v1.or(&v2).and(&bv).bits().collect::<Vec<u32>>());
}

#[bench]
fn bitset_and_not(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let v1 = gen_bitset!(SIZE, RANGE1, rng);
    let v2 = gen_bitset!(SIZE, RANGE2, rng);
    let bv = gen_bitset!(0, 1, rng);
    bench.iter(|| v1.and_not(&v2).and(&bv).bits().collect::<Vec<u32>>());
}

#[bench]
fn bitset_xor(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let v1 = gen_bitset!(SIZE, RANGE1, rng);
    let v2 = gen_bitset!(SIZE, RANGE2, rng);
    let bv = gen_bitset!(0, 1, rng);
    bench.iter(|| v1.xor(&v2).and(&bv).bits().collect::<Vec<u32>>());
}

#[bench]
fn bitset_rank_small(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let v1 = gen_bitset!(SIZE, RANGE1, rng);
    let i = rng.gen();
    bench.iter(|| v1.rank1(i));
}
#[bench]
fn bitset_rank_large(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let v1 = gen_bitset!(SIZE, RANGE2, rng);
    let i = rng.gen();
    bench.iter(|| v1.rank1(i));
}

#[bench]
fn bitset_optimize(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let mut set = gen_bitset!(SIZE, RANGE2, rng);
    bench.iter(|| set.optimize());
}

#[bench]
fn bitset_read_from_file_withruns(bench: &mut Bencher) {
    bench.iter(|| {
        let mut file = fs::File::open("./tests/bitmapwithruns.bin").unwrap();
        BitSet::read_from(&mut file).unwrap()
    });
}

#[bench]
fn bitset_read_from_buff_withruns(bench: &mut Bencher) {
    bench.iter(|| {
        let file = fs::File::open("./tests/bitmapwithruns.bin").unwrap();
        BitSet::read_from(&mut io::BufReader::new(file)).unwrap()
    });
}

#[bench]
fn bitset_read_from_file_withoutruns(bench: &mut Bencher) {
    bench.iter(|| {
        let mut file = fs::File::open("./tests/bitmapwithoutruns.bin").unwrap();
        BitSet::read_from(&mut file).unwrap()
    });
}

#[bench]
fn bitset_read_from_buff_withoutruns(bench: &mut Bencher) {
    bench.iter(|| {
        let file = fs::File::open("./tests/bitmapwithoutruns.bin").unwrap();
        BitSet::read_from(&mut io::BufReader::new(file)).unwrap()
    });
}

fn test_set() -> BitSet {
    let mut set = BitSet::new();
    for i in 0..100_000 {
        if i % 1000 == 0 {
            set.put(i, true);
        }
    }
    for i in 100_000..200_000 {
        set.put(i * 3, true);
    }
    for i in 700_000..800_000 {
        set.put(i, true);
    }
    set.optimize();
    set
}

#[bench]
fn bitset_write_to_buff(bench: &mut Bencher) {
    let set = test_set();
    let mut n = 0;
    let mut w = Vec::with_capacity(1 << 16);
    bench.iter(|| {
        set.write_to(&mut w).unwrap();
        n += 1;
    });
    // print!("{:>8} {:>12} ", n, w.len())
}

#[bench]
fn bitset_write_to_buff_snap(bench: &mut Bencher) {
    use std::io::Write;

    let set = test_set();
    let mut n = 0;
    let mut w = Vec::with_capacity(1 << 16);
    {
        let mut buf = snap::Writer::new(&mut w);
        bench.iter(|| {
            set.write_to(&mut buf).unwrap();
            n += 1;
        });
        buf.flush().unwrap();
    }
    // print!("{:>8} {:>12} ", n, w.len())
}

#[bench]
fn bitset_write_to_buff_zstd(bench: &mut Bencher) {
    let set = test_set();
    let mut n = 0;
    let mut w = Vec::with_capacity(1 << 16);
    {
        let mut buf = zstd::Encoder::new(&mut w, 0).unwrap();
        bench.iter(|| {
            set.write_to(&mut buf).unwrap();
            n += 1;
        });
        buf.finish().unwrap();
    }
    // print!("{:>8} {:>12} ", n, w.len())
}

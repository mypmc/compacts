#![feature(test)]

extern crate compacts;
extern crate rand;
extern crate snap;
extern crate test;
extern crate zstd;

use std::{fs, io};
use test::Bencher;
use rand::Rng;
use compacts::bits::*;
use compacts::{ReadFrom, WriteTo};

macro_rules! gen_bitset {
    ( 0, 1, $rng:expr ) => {
        {
            Set::new()
        }
    };
    ( $size:expr, $end:expr, $rng:expr ) => {
        {
            gen_bitset!($size, 0, $end, $rng)
        }
    };
    ( $size:expr, $start:expr, $end:expr, $rng:expr ) => {
        {
            let mut bs = Set::new();
            if $size > 1 {
                for _ in 0..$size {
                    let gen = $rng.gen_range($start, $end);
                    bs.insert(gen);
                }
                bs.optimize();
            }
            bs
        }
    };
}

const SIZE: usize = 65_000;
const RANGE1: u32 = 150_000;
const RANGE2: u32 = 100_000_000;

#[bench]
fn set_contains(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let v1 = gen_bitset!(SIZE, RANGE1, rng);
    bench.iter(|| v1.contains(rng.gen()));
}

#[bench]
fn set_insert(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let mut v1 = gen_bitset!(SIZE, RANGE1, rng);
    bench.iter(|| v1.insert(rng.gen()));
}

#[bench]
fn set_remove(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let mut v1 = gen_bitset!(SIZE, RANGE1, rng);
    bench.iter(|| v1.remove(rng.gen()));
}

#[bench]
fn set_bits(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let v1 = gen_bitset!(SIZE, RANGE1, rng);
    bench.iter(|| v1.bits().collect::<Vec<u32>>());
}

#[bench]
fn set_clone_small(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let mut v1 = gen_bitset!(SIZE, RANGE1, rng);
    bench.iter(|| v1 = v1.clone());
}
#[bench]
fn set_clone_large(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let mut v1 = gen_bitset!(SIZE, RANGE2, rng);
    bench.iter(|| v1 = v1.clone());
}


#[bench]
fn set_and(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let v1 = gen_bitset!(SIZE, RANGE1, rng);
    let v2 = gen_bitset!(SIZE, RANGE2, rng);
    bench.iter(|| v1.and(&v2));
}

#[bench]
fn set_or(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let v1 = gen_bitset!(SIZE, RANGE1, rng);
    let v2 = gen_bitset!(SIZE, RANGE2, rng);
    let bv = gen_bitset!(0, 1, rng);
    bench.iter(|| v1.or(&v2).and(&bv).bits().collect::<Vec<u32>>());
}

#[bench]
fn set_and_not(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let v1 = gen_bitset!(SIZE, RANGE1, rng);
    let v2 = gen_bitset!(SIZE, RANGE2, rng);
    let bv = gen_bitset!(0, 1, rng);
    bench.iter(|| v1.and_not(&v2).and(&bv).bits().collect::<Vec<u32>>());
}

#[bench]
fn set_xor(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let v1 = gen_bitset!(SIZE, RANGE1, rng);
    let v2 = gen_bitset!(SIZE, RANGE2, rng);
    let bv = gen_bitset!(0, 1, rng);
    bench.iter(|| v1.xor(&v2).and(&bv).bits().collect::<Vec<u32>>());
}

#[bench]
fn set_rank_small(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let v1 = gen_bitset!(SIZE, RANGE1, rng);
    let i = rng.gen();
    bench.iter(|| v1.rank1(i));
}
#[bench]
fn set_rank_large(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let v1 = gen_bitset!(SIZE, RANGE2, rng);
    let i = rng.gen();
    bench.iter(|| v1.rank1(i));
}

#[bench]
fn set_optimize(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let mut set = gen_bitset!(SIZE, RANGE2, rng);
    bench.iter(|| set.optimize());
}

#[bench]
fn set_read_from_file_withruns(bench: &mut Bencher) {
    bench.iter(|| {
        let mut file = fs::File::open("./tests/bitmapwithruns.bin").unwrap();
        let mut bits = Set::new();
        bits.read_from(&mut file).unwrap()
    });
}

#[bench]
fn set_read_from_buff_withruns(bench: &mut Bencher) {
    bench.iter(|| {
        let file = fs::File::open("./tests/bitmapwithruns.bin").unwrap();
        let mut bits = Set::new();
        bits.read_from(&mut io::BufReader::new(file)).unwrap()
    });
}

#[bench]
fn set_read_from_file_withoutruns(bench: &mut Bencher) {
    bench.iter(|| {
        let mut file = fs::File::open("./tests/bitmapwithoutruns.bin").unwrap();
        let mut bits = Set::new();
        bits.read_from(&mut file).unwrap()
    });
}

#[bench]
fn set_read_from_buff_withoutruns(bench: &mut Bencher) {
    bench.iter(|| {
        let file = fs::File::open("./tests/bitmapwithoutruns.bin").unwrap();
        let mut bits = Set::new();
        bits.read_from(&mut io::BufReader::new(file)).unwrap()
    });
}

fn test_set() -> Set {
    let mut set = Set::new();
    for i in 0..100_000 {
        if i % 1000 == 0 {
            set.insert(i);
        }
    }
    for i in 100_000..200_000 {
        set.insert(i * 3);
    }
    for i in 700_000..800_000 {
        set.insert(i);
    }
    set.optimize();
    set
}

#[bench]
fn set_write_to_buff(bench: &mut Bencher) {
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
fn set_write_to_buff_snap(bench: &mut Bencher) {
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
fn set_write_to_buff_zstd(bench: &mut Bencher) {
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

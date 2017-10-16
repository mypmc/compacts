#![feature(test)]

extern crate compacts;
extern crate rand;
extern crate test;

use std::{fs, io};
use test::Bencher;
use rand::Rng;
use compacts::bits::*;
use compacts::{ReadFrom, WriteTo};

macro_rules! bit_vec {
    ( 0, 1, $rng:expr ) => {
        { Map::new() }
    };
    ( $size:expr, $end:expr, $rng:expr ) => {
        {
            bit_vec!($size, 0, $end, $rng)
        }
    };
    ( $size:expr, $start:expr, $end:expr, $rng:expr ) => {
        {
            let mut vec = Map::new();
            if $size > 1 {
                for _ in 0..$size {
                    let gen = $rng.gen_range($start, $end);
                    vec.insert(gen);
                }
                vec.optimize();
            }
            vec
        }
    };
}

const SIZE: usize = 65_000;
const RANGE1: u32 = 150_000;
const RANGE2: u32 = 100_000_000;

#[bench]
fn contains(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let v1 = bit_vec!(SIZE, RANGE1, rng);
    bench.iter(|| v1.contains(rng.gen()));
}

#[bench]
fn insert(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let mut v1 = bit_vec!(SIZE, RANGE1, rng);
    bench.iter(|| v1.insert(rng.gen()));
}

#[bench]
fn remove(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let mut v1 = bit_vec!(SIZE, RANGE1, rng);
    bench.iter(|| v1.remove(rng.gen()));
}

#[bench]
fn clone(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let mut v1 = bit_vec!(SIZE, RANGE1, rng);
    bench.iter(|| v1 = v1.clone());
}

#[bench]
fn and(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let v1 = bit_vec!(SIZE, RANGE1, rng);
    let v2 = bit_vec!(SIZE, RANGE2, rng);
    bench.iter(|| v1.and(&v2));
}

#[bench]
fn or(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let v1 = bit_vec!(SIZE, RANGE1, rng);
    let v2 = bit_vec!(SIZE, RANGE2, rng);
    let bv = bit_vec!(0, 1, rng);
    bench.iter(|| v1.or(&v2).and(&bv).bits().collect::<Vec<u32>>());
}

#[bench]
fn and_not(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let v1 = bit_vec!(SIZE, RANGE1, rng);
    let v2 = bit_vec!(SIZE, RANGE2, rng);
    let bv = bit_vec!(0, 1, rng);
    bench.iter(|| v1.and_not(&v2).and(&bv).bits().collect::<Vec<u32>>());
}

#[bench]
fn xor(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let v1 = bit_vec!(SIZE, RANGE1, rng);
    let v2 = bit_vec!(SIZE, RANGE2, rng);
    let bv = bit_vec!(0, 1, rng);
    bench.iter(|| v1.xor(&v2).and(&bv).bits().collect::<Vec<u32>>());
}

#[bench]
fn rank_small(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let v1 = bit_vec!(SIZE, RANGE1, rng);
    let i = rng.gen();
    bench.iter(|| v1.rank1(i));
}

#[bench]
fn rank_large(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let v1 = bit_vec!(SIZE, RANGE2, rng);
    let i = rng.gen();
    bench.iter(|| v1.rank1(i));
}

#[bench]
fn read_from_file_withruns(bench: &mut Bencher) {
    bench.iter(|| {
        let mut file = fs::File::open("./tests/bitmapwithruns.bin").unwrap();
        let mut bits = Map::new();
        bits.read_from(&mut file).unwrap()
    });
}

#[bench]
fn read_from_buff_withruns(bench: &mut Bencher) {
    bench.iter(|| {
        let file = fs::File::open("./tests/bitmapwithruns.bin").unwrap();
        let mut bits = Map::new();
        bits.read_from(&mut io::BufReader::new(file)).unwrap()
    });
}

#[bench]
fn read_from_file_withoutruns(bench: &mut Bencher) {
    bench.iter(|| {
        let mut file = fs::File::open("./tests/bitmapwithoutruns.bin").unwrap();
        let mut bits = Map::new();
        bits.read_from(&mut file).unwrap()
    });
}

#[bench]
fn read_from_buff_withoutruns(bench: &mut Bencher) {
    bench.iter(|| {
        let file = fs::File::open("./tests/bitmapwithoutruns.bin").unwrap();
        let mut bits = Map::new();
        bits.read_from(&mut io::BufReader::new(file)).unwrap()
    });
}

#[bench]
fn write_to_buff(bench: &mut Bencher) {
    let map = {
        let mut map = Map::new();
        for i in 0..100000 {
            if i % 1000 == 0 {
                map.insert(i);
            }
        }
        for i in 100000..200000 {
            map.insert(i * 3);
        }
        for i in 700000..800000 {
            map.insert(i);
        }
        map.optimize();
        map
    };
    let mut buf = Vec::with_capacity(8192);

    bench.iter(|| map.write_to(&mut buf).unwrap());
}

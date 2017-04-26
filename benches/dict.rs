#![feature(test)]

extern crate cds;
extern crate rand;
extern crate test;
use rand::Rng;

use cds::dict::{Dict, Ranked, Select1, Bits};
use cds::prim::TRUE;

#[bench]
fn dict_count1(bench: &mut test::Bencher) {
    let mut rng = rand::thread_rng();
    let bits = Bits::new(rng.gen());
    let ranked = &bits as &Ranked<u32, Weight = u32>;
    bench.iter(|| ranked.count1());
}

#[bench]
fn dict_select(bench: &mut test::Bencher) {
    let mut rng = rand::thread_rng();
    let bits = Bits::new(rng.gen());
    let ranked = &bits as &Ranked<u32, Weight = u32>;
    let pop: u32 = ranked.count1();
    bench.iter(|| bits.select(TRUE, pop / 2));
}

#[bench]
fn dict_select1(bench: &mut test::Bencher) {
    let mut rng = rand::thread_rng();
    let bits = Bits::new(rng.gen());
    let ranked = &bits as &Ranked<u32, Weight = u32>;
    let pop: u32 = ranked.count1();
    bench.iter(|| bits.select1(pop / 2));
}

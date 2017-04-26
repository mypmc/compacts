#[macro_use]
extern crate log;
extern crate env_logger;
extern crate rand;
extern crate cds;

use rand::Rng;
use cds::BitVec;
use cds::bits::PairwiseWith;

macro_rules! bit_vec {
    ( ) => {&BitVec::new()};

    ( $size:expr, $end:expr, $rng:expr ) => {{
        bit_vec!($size, 0, $end, $rng)
    }};
    ( $size:expr, $start:expr, $end:expr, $rng:expr ) => {{
        let mut vec = BitVec::new();
        for _ in 0..$size {
            let gen = $rng.gen_range($start, $end);
            vec.insert(gen);
        }
        vec
    }};
}

const SIZE: usize = 10000;
const RANGE: u32 = 10000000;

#[test]
fn similarity_coefficient() {
    let _ = env_logger::init();
    let mut rng = rand::thread_rng();

    let size = 1000;
    let range = 3000;

    let ref p = bit_vec!(size, range, rng);
    let ref q = bit_vec!(size, range, rng);

    let jaccard = {
        let mut p1 = p.clone();
        p1.intersection_with(q);
        p1.count1() as f64 / (p.count1() + q.count1() + p1.count1()) as f64
    };

    let dice = {
        let mut p1 = p.clone();
        p1.intersection_with(q);
        (2f64 * (p1.count1() as f64)) / (p.count1() as f64 + q.count1() as f64)
    };

    let simpson = {
        let mut p1 = p.clone();
        p1.intersection_with(q);
        (p1.count1() as f64) / (p.count1() as f64).min(q.count1() as f64)
    };

    info!("Jaccard = {:.5?}", jaccard);
    info!("Dice    = {:.5?}", dice);
    info!("Simpson = {:.5?}", simpson);

    info!("JaccardDistance = {:.5?}", 1f64 - jaccard);
}

macro_rules! pairwise_do {
    ( $e:expr ) => {
        let _ = env_logger::init();
        let mut rng = rand::thread_rng();
        let mut v1 = bit_vec!(SIZE, RANGE, rng);
        let ref v2 = bit_vec!(SIZE, RANGE, rng);
        let ref v3 = bit_vec!(SIZE, RANGE, rng);
        let ref v4 = bit_vec!(SIZE, RANGE, rng);
        let ref v5 = bit_vec!(SIZE, RANGE, rng);

        if $e {
            trace!("This test should not cause any evaluations");
        }

        trace!("INTERSECTION does not deferred");
        v1.intersection_with(v2);
        if $e {v1.intersection_with(bit_vec!());}

        trace!("UNION may force evaluation of blocks that already deferred");
        v1.union_with(v3);
        if $e {v1.intersection_with(bit_vec!());}

        trace!("DIFFERENCE may force evaluation of blocks that already deferred");
        v1.difference_with(v4);
        if $e {v1.intersection_with(bit_vec!());}

        trace!("SYMMETRIC_DIFFERENCE may force evaluation of blocks that already deferred");
        v1.symmetric_difference_with(v5);
        if $e {v1.intersection_with(bit_vec!());}

        trace!("COUNT_BLOCKS should NOT evaluate thunks");
        trace!("COUNT_BLOCKS={:?}", v1.count_blocks());
        if !$e {
            trace!("POP_COUNT force evaluation of thunks");
            trace!("POP_COUNT={:?}", v1.count1());
        }
    }
}

#[test]
fn pairwise_no_interleave() {
    // To see evaluation progress
    // RUST_LOG=thunk=trace,cds=trace cargo test
    pairwise_do!(false);
}

#[test]
fn pairwise_interleave() {
    // To see evaluation progress
    // RUST_LOG=thunk=trace,cds=trace cargo test
    pairwise_do!(true);
}

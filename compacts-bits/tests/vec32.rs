#[macro_use]
extern crate log;
extern crate env_logger;
extern crate rand;
extern crate compacts_bits;

use self::rand::Rng;

use compacts_bits::*;
use compacts_bits::ops::*;

macro_rules! bit_vec {
    ( ) => {&Vec32::new()};

    ( $size:expr, $end:expr, $rng:expr ) => {{
        bit_vec!($size, 0, $end, $rng)
    }};
    ( $size:expr, $start:expr, $end:expr, $rng:expr ) => {{
        let mut vec = Vec32::new();
        for _ in 0..$size {
            let gen = $rng.gen_range($start, $end);
            vec.insert(gen);
        }
        vec.optimize();
        vec
    }};
}

#[test]
#[ignore]
fn similarity() {
    let _ = env_logger::init();
    let mut rng = rand::thread_rng();

    let size = (1 << 15) * 7;
    let maxn = (1 << 16) * 2;

    let p = &(bit_vec!(size, maxn, rng));
    let q = &(bit_vec!(size, maxn, rng));

    debug!("{:#?}", p.stats().sum::<vec32::Summary>());

    let jaccard = {
        let r = p.intersection(q);
        r.count_ones() as f64 / (p.count_ones() + q.count_ones() - r.count_ones()) as f64
    };

    let dice = {
        let r = p.intersection(q);
        (2.0 * (r.count_ones() as f64)) / (p.count_ones() + q.count_ones()) as f64
    };

    let simpson = {
        let r = p.intersection(q);
        (r.count_ones() as f64) / (p.count_ones() as f64).min(q.count_ones() as f64)
    };

    info!("Jaccard  = {:.5?}", jaccard);
    info!("Dice     = {:.5?}", dice);
    info!("Simpson  = {:.5?}", simpson);
    info!("Distance = {:.5?}", 1f64 - jaccard);
}

#[test]
fn rank_select() {
    use compacts_bits::{Rank, Select1, Select0};

    let _ = env_logger::init();
    let mut vec = Vec32::new();
    vec.insert(0);
    vec.insert(1000000);

    assert_eq!(vec.select0(0), Some(1));
    assert_eq!(vec.rank0(1), 1);

    assert_eq!(vec.select1(0), Some(0));
    assert_eq!(vec.rank1(0), 1);

    assert_eq!(vec.select0(1), Some(2));
    assert_eq!(vec.rank0(2), 2);

    assert_eq!(vec.select1(1), Some(1000000));
    assert_eq!(vec.rank1(1000000), 2);
}

#[test]
fn iterator() {
    let _ = env_logger::init();

    {
        let mut vec = Vec32::new();
        for i in 0..1000000 {
            vec.insert(i);
        }
        for (i, bit) in vec.iter().enumerate() {
            assert_eq!(i as u32, bit);
        }
    }
    {
        let mut vec = Vec32::new();
        for i in 65533..65537 {
            vec.insert(i);
        }
        let col = vec.iter().collect::<Vec<u32>>();
        assert_eq!(col, vec![65533, 65534, 65535, 65536]);
    }
}

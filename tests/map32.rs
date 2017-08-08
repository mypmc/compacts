extern crate compacts;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate rand;

#[macro_use]
mod helper;

use compacts::bits::*;
use self::rand::Rng;

#[test]
#[ignore]
fn similarity() {
    let _ = env_logger::init();
    let mut rng = rand::thread_rng();

    let size = (1 << 15) * 7;
    let maxn = (1 << 16) * 2;

    let p = &(bit_vec!(size, maxn, rng));
    let q = &(bit_vec!(size, maxn, rng));

    debug!("{:#?}", p.stats().sum::<Summary>());

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
    use compacts::dict::{Rank, Select0, Select1};

    let _ = env_logger::init();
    let mut vec = Map32::new();
    vec.insert(0);
    vec.insert(65536);
    vec.insert(1_000_000);

    assert_eq!(vec.rank0(0), 0);
    assert_eq!(vec.rank0(1), 0);
    assert_eq!(vec.rank0(2), 1);
    assert_eq!(vec.rank0(65536), 65535);
    assert_eq!(vec.rank0(1_000_000), 999_998);

    assert_eq!(vec.rank1(0), 0);
    assert_eq!(vec.rank1(1), 1);
    assert_eq!(vec.rank1(2), 1);
    assert_eq!(vec.rank1(65536), 1);
    assert_eq!(vec.rank1(1_000_000), 2);

    assert_eq!(vec.select0(0), Some(1));
    assert_eq!(vec.select0(1), Some(2));
    assert_eq!(vec.select0(2), Some(3));

    assert_eq!(vec.select1(0), Some(0));
    assert_eq!(vec.select1(1), Some(65536));
    assert_eq!(vec.select1(2), Some(1_000_000));
    assert_eq!(vec.select1(3), None);
}

#[test]
fn iterator() {
    let _ = env_logger::init();

    {
        let mut vec = Map32::new();
        for i in 0..1_000_000 {
            vec.insert(i);
        }
        for (i, bit) in vec.iter().enumerate() {
            assert_eq!(i as u32, bit);
        }
    }
    {
        let mut vec = Map32::new();
        for i in 65_533..65_537 {
            vec.insert(i);
        }
        let col = vec.iter().collect::<Vec<u32>>();
        assert_eq!(col, vec![65_533, 65_534, 65_535, 65_536]);
    }
}

#[test]
fn intersection() {
    let _ = env_logger::init();

    let mut b1 = {
        let mut vec = Map32::new();
        vec.insert(1 << 16);
        vec.insert(1 << 20);
        vec
    };
    let b2 = {
        let mut vec = Map32::new();
        vec.insert(1 << 10);
        vec.insert(1 << 11);
        vec.insert(1 << 20);
        vec
    };

    b1.intersection_with(&b2);
    debug!("{:?} {:?}", b1, b2);
    assert_eq!(b1.count_ones(), 1);
}

#[test]
fn union() {
    let _ = env_logger::init();

    let mut b1 = {
        let mut vec = Map32::new();
        vec.insert(1 << 16);
        vec.insert(1 << 20);
        vec
    };

    let b2 = {
        let mut vec = Map32::new();
        vec.insert(1 << 10);
        vec.insert(1 << 11);
        vec.insert(1 << 20);
        vec
    };

    b1.union_with(&b2);
    debug!("{:?} {:?}", b1, b2);
    assert_eq!(b1.count_ones(), 4);
}

#[test]
fn difference() {
    let _ = env_logger::init();

    let mut b1 = {
        let mut vec = Map32::new();
        vec.insert(1 << 10);
        vec.insert(1 << 11);
        vec.insert(1 << 12);
        vec.insert(1 << 16);
        vec.insert(1 << 20);
        vec
    };
    let b2 = {
        let mut vec = Map32::new();
        vec.insert(1 << 10);
        vec.insert(1 << 11);
        vec.insert(1 << 20);
        vec
    };

    b1.difference_with(&b2);
    debug!("{:?} {:?}", b1, b2);
    assert_eq!(b1.count_ones(), 2);
}

#[test]
fn symmetric_difference() {
    let _ = env_logger::init();

    let mut b1 = {
        let mut vec = Map32::new();
        vec.insert(1 << 10);
        vec.insert(1 << 11);
        vec.insert(1 << 12);
        vec.insert(1 << 16);
        vec.insert(1 << 20);
        vec
    };
    let b2 = {
        let mut vec = Map32::new();
        vec.insert(1 << 10);
        vec.insert(1 << 11);
        vec.insert(1 << 20);
        vec.insert(1 << 26);
        vec.insert(1 << 30);
        vec
    };

    b1.symmetric_difference_with(&b2);
    debug!("{:?} {:?}", b1, b2);
    assert_eq!(b1.count_ones(), 4);
}

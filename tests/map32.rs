extern crate compacts;
extern crate env_logger;
#[macro_use]
extern crate log;

use compacts::bits::*;
use compacts::dict::PopCount;

// #[test]
// #[ignore]
// fn similarity() {
//     let _ = env_logger::init();
//     let mut rng = rand::thread_rng();
//
//     let size = (1 << 15) * 7;
//     let maxn = (1 << 16) * 2;
//
//     let p = &(bit_vec!(size, maxn, rng));
//     let q = &(bit_vec!(size, maxn, rng));
//
//     let jaccard = {
//         let r = p.intersection(q);
//         r.count_ones() as f64 / (p.count_ones() + q.count_ones() - r.count_ones()) as f64
//     };
//
//     let dice = {
//         let r = p.intersection(q);
//         (2.0 * (r.count_ones() as f64)) / (p.count_ones() + q.count_ones()) as f64
//     };
//
//     let simpson = {
//         let r = p.intersection(q);
//         (r.count_ones() as f64) / (p.count_ones() as f64).min(q.count_ones() as f64)
//     };
//
//     info!("Jaccard  = {:.5?}", jaccard);
//     info!("Dice     = {:.5?}", dice);
//     info!("Simpson  = {:.5?}", simpson);
//     info!("Distance = {:.5?}", 1f64 - jaccard);
// }

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
    assert_eq!(b1.count1(), 1);
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
    assert_eq!(b1.count1(), 4);
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
    assert_eq!(b1.count1(), 2);
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
    assert_eq!(b1.count1(), 4);
}

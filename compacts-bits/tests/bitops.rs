#[macro_use]
extern crate log;
extern crate env_logger;
extern crate compacts_bits;

use compacts_bits::Vec32;
use compacts_bits::ops::*;

#[test]
fn intersection() {
    let _ = env_logger::init();

    let mut b1 = {
        let mut vec = Vec32::new();
        vec.insert(1 << 16);
        vec.insert(1 << 20);
        vec
    };
    let b2 = {
        let mut vec = Vec32::new();
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
        let mut vec = Vec32::new();
        vec.insert(1 << 16);
        vec.insert(1 << 20);
        vec
    };

    let b2 = {
        let mut vec = Vec32::new();
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
        let mut vec = Vec32::new();
        vec.insert(1 << 10);
        vec.insert(1 << 11);
        vec.insert(1 << 12);
        vec.insert(1 << 16);
        vec.insert(1 << 20);
        vec
    };
    let b2 = {
        let mut vec = Vec32::new();
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
        let mut vec = Vec32::new();
        vec.insert(1 << 10);
        vec.insert(1 << 11);
        vec.insert(1 << 12);
        vec.insert(1 << 16);
        vec.insert(1 << 20);
        vec
    };
    let b2 = {
        let mut vec = Vec32::new();
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

#![feature(test)]

extern crate rand;
extern crate test;

use test::Bencher;
use rand::Rng;

const SIZE: usize = 65_000;

#[bench]
fn repr_map_iterate(bench: &mut Bencher) {
    use std::collections::BTreeMap;
    let mut b = BTreeMap::new();
    for i in 0..SIZE {
        b.insert(i, ());
    }
    bench.iter(|| b.keys().collect::<Vec<&usize>>());
}

#[bench]
fn repr_vec_iterate(bench: &mut Bencher) {
    let mut b = Vec::with_capacity(SIZE);
    for i in 0..SIZE {
        b.push((i, ()));
    }
    bench.iter(|| b.iter().map(|&(k, _)| k).collect::<Vec<usize>>());
}

#[bench]
fn repr_zip_iterate(bench: &mut Bencher) {
    let mut b1 = Vec::with_capacity(SIZE);
    let mut b2 = Vec::with_capacity(SIZE);
    for i in 0..SIZE {
        b1.push(i);
        b2.push(i);
    }
    bench.iter(|| {
        b1.iter().zip(&b2).map(|(&k, _)| k).collect::<Vec<usize>>()
    });
}

#[bench]
fn repr_map_insert(bench: &mut Bencher) {
    use std::collections::BTreeMap;
    let mut rng = rand::thread_rng();
    let mut b = BTreeMap::new();
    bench.iter(|| {
        let n = rng.gen::<u16>();
        b.insert(n, ());
    });
}

#[bench]
fn repr_vec_insert(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let mut b = Vec::new();
    bench.iter(|| {
        let n = rng.gen::<u16>();
        b.binary_search_by_key(&n, |&(k, _)| k)
            .map_err(|i| b.insert(i, (n, ())))
    });
}

#[bench]
fn repr_zip_insert(bench: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let mut b1 = Vec::new();
    let mut b2 = Vec::new();
    bench.iter(|| {
        let n = rng.gen::<u16>();
        b1.binary_search(&n).map_err(|i| {
            b1.insert(i, n);
            b2.insert(i, n);
        })
    });
}

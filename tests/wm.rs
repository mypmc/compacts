#[allow(unused_imports)]
use {
    compacts::{
        bits::{and, or, xor, Fold, Mask},
        ops::*,
        BitArray, BitMap, WaveletMatrix,
    },
    lazy_static::lazy_static,
    quickcheck::quickcheck,
    rand::prelude::*,
};

quickcheck! {
    fn index_all(vec: Vec<u64>) -> bool {
        let mut xs = vec.clone();
        let wm = WaveletMatrix::<u64, BitArray<u64>>::from(&mut xs[..]);
        vec.iter().enumerate().all(|(i, v)| wm.get(i).unwrap() == *v)
    }
}

#![allow(unused_features)]
#![feature(associated_consts)]
#![feature(box_patterns)]
#![feature(test)]

/// Inspired from:
///   [Broadword implementation of rank/select queries](http://sux.di.unimi.it/paper.pdf);
///   Springer Berlin Heidelberg, 2008. 154-168.

mod pop_count;
mod rank;
mod select;
mod bucket;
mod bits;

mod bit_vec;

pub use pop_count::PopCount;
pub use rank::{Rank0, Rank1};
pub use select::{Select0, Select1};
pub use bit_vec::BitVec;


/* Private API */

use pop_count::Bounded;
use bucket::Bucket;
use bucket::Iter as BucketIter;

mod dir {
    pub trait Direction {}

    #[derive(Debug, Clone)]
    pub struct Forward;
    impl Direction for Forward {}

    //#[derive(Debug, Clone)]
    //pub struct Reverse;
    //impl Direction for Reverse {}
}

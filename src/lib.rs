#![allow(unused_features)]
#![feature(associated_consts)]
#![feature(test)]

/// Inspired from:
///   [Broadword implementation of rank/select queries](http://sux.di.unimi.it/paper.pdf);
///   Springer Berlin Heidelberg, 2008. 154-168.

mod pop_count;
mod rank;
mod select;
mod bucket;
mod bit_map;
mod bits;

pub use pop_count::PopCount;
pub use rank::{Rank0, Rank1};
pub use select::{Select0, Select1};
pub use bit_map::BitMap;


/* Private API */

use pop_count::Bounded;
use bucket::Bucket;
use bucket::Iter as BucketIter;

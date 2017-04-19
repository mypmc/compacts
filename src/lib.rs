#![feature(associated_consts)]
#![feature(test)]

/// Inspired from:
///   [Broadword implementation of rank/select queries](http://sux.di.unimi.it/paper.pdf);
///   Springer Berlin Heidelberg, 2008. 154-168.

extern crate test;

mod bits;
pub use bits::{Bits, Count};
pub use bits::{Bounded, SplitMerge};

mod rank;
pub use rank::{Rank0, Rank1};

mod select;
pub use select::{Select0, Select1};

mod bucket;
use bucket::{Bucket, Iter as BucketIter};

mod bit_map;
pub use bit_map::BitMap;

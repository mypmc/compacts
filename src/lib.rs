#![feature(associated_consts)]
#![feature(test)]

/// Inspired from:
///   [Broadword implementation of rank/select queries](http://sux.di.unimi.it/paper.pdf);
///   Springer Berlin Heidelberg, 2008. 154-168.

extern crate test;

mod bits;
pub use bits::Bits;

mod rank;
pub use rank::{Rank0, Rank1};

mod select;
pub use select::{Select0, Select1};

mod pop_count;
use pop_count::{Bounded, PopCount};

mod repr;
use repr::{Repr, Iter};

mod bit_map;
use bit_map::BitMap;

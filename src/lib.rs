#![feature(associated_consts)]
#![feature(test)]

/// Inspired from:
///   [Broadword implementation of rank/select queries](http://sux.di.unimi.it/paper.pdf);
///   Springer Berlin Heidelberg, 2008. 154-168.

extern crate test;

mod bits;
pub use bits::Bits;
pub use bits::{Rank0, Rank1};
pub use bits::{Select0, Select1};

mod repr;
use repr::{Repr, Iter};

mod bit_map;

#![feature(associated_consts)]
#![feature(conservative_impl_trait)]
#![feature(inclusive_range)]
#![feature(inclusive_range_syntax)]
#![feature(i128_type)]

// #![deny(warnings)]

#[macro_use]
extern crate karabiner;

extern crate itertools;

extern crate compacts_prim as prim;

// Broadword implementation of rank/select queries
// (http://sux.di.unimi.it/paper.pdf);
// Springer Berlin Heidelberg, 2008. 154-168.

#[macro_use]
mod macros;
mod block;
mod inner;
mod split_merge;

mod pairwise;
mod rank;
mod select;
mod bit_vec;
mod bit_map;

pub use bit_vec::BitVec;
pub use bit_map::BitMap;
pub use rank::Rank;
pub use select::{Select0, Select1};

pub mod ops {
    pub use pairwise::{Intersection, IntersectionWith};
    pub use pairwise::{Union, UnionWith};
    pub use pairwise::{Difference, DifferenceWith};
    pub use pairwise::{SymmetricDifference, SymmetricDifferenceWith};
}

pub static TRUE: &bool = &true;
pub static FALSE: &bool = &false;

pub(crate) use block::Block;
pub(crate) use split_merge::{Split, Merge};
pub(crate) use prim::{UnsignedInt, Zero};
// pub(crate) use inner::{Seq16, Seq64, Rle16};

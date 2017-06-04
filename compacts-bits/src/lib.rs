#![feature(associated_consts)]
#![feature(conservative_impl_trait)]
#![feature(inclusive_range)]
#![feature(inclusive_range_syntax)]

#![deny(warnings)]

#[macro_use]
extern crate karabiner;

extern crate itertools;

extern crate compacts_prim as prim;

// Broadword implementation of rank/select queries
// (http://sux.di.unimi.it/paper.pdf);
// Springer Berlin Heidelberg, 2008. 154-168.

#[macro_use]
mod macros;
mod bit_vec;
mod block;
mod rank;
mod select;
mod split_merge;
mod pairwise;

pub use bit_vec::BitVec;
pub use rank::Rank;
pub use select::{Select0, Select1};

pub mod ops {
    pub use super::pairwise::{Intersection, IntersectionWith};
    pub use super::pairwise::{Union, UnionWith};
    pub use super::pairwise::{Difference, DifferenceWith};
    pub use super::pairwise::{SymmetricDifference, SymmetricDifferenceWith};
}

pub static TRUE: &bool = &true;
pub static FALSE: &bool = &false;

use prim::UnsignedInt;

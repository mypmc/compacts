#![feature(associated_consts)]
#![feature(conservative_impl_trait)]
#![feature(inclusive_range)]
#![feature(inclusive_range_syntax)]
#![feature(i128_type)]

// #![deny(warnings)]

#[macro_use]
extern crate karabiner;
#[macro_use]
extern crate compacts_prim as prim;
extern crate itertools;

// Broadword implementation of rank/select queries
// ( http://sux.di.unimi.it/paper.pdf );
// Springer Berlin Heidelberg, 2008. 154-168.

#[macro_use]
mod macros;
mod split_merge;
mod pairwise;
mod rank;
mod select;

mod block; // internal representaions of vec16::Vec16
mod vec16; // bit vector of u16
pub mod vec32; // bit vector of u32
pub mod vec64; // bit vector of u64

pub mod ops {
    pub use pairwise::{Intersection, IntersectionWith};
    pub use pairwise::{Union, UnionWith};
    pub use pairwise::{Difference, DifferenceWith};
    pub use pairwise::{SymmetricDifference, SymmetricDifferenceWith};
}

pub static TRUE: &bool = &true;
pub static FALSE: &bool = &false;

pub(crate) use prim::UnsignedInt;
pub(crate) use split_merge::{Split, Merge};
pub(crate) use vec16::Vec16;
pub use vec32::Vec32;
pub use vec64::Vec64;
pub use rank::Rank;
pub use select::{Select0, Select1};

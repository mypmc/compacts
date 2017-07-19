#![feature(conservative_impl_trait)]
#![feature(inclusive_range)]
#![feature(inclusive_range_syntax)]
#![feature(i128_type)]

// #![deny(warnings)]

extern crate compacts_prim as prim;

#[macro_use]
extern crate karabiner;
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

mod block;
mod vec16; // (u16) bit vector
mod vec32; // (u32) bit vector
mod vec64; // (u64) bit vector

mod ops {
    pub use pairwise::{Intersection, IntersectionWith};
    pub use pairwise::{Union, UnionWith};
    pub use pairwise::{Difference, DifferenceWith};
    pub use pairwise::{SymmetricDifference, SymmetricDifferenceWith};
}
pub use ops::*;

pub static TRUE: &bool = &true;
pub static FALSE: &bool = &false;

pub(crate) use prim::UnsignedInt;
pub(crate) use split_merge::{Split, Merge};
pub(crate) use vec16::Vec16;
pub use vec32::Vec32;
pub use vec64::Vec64;
pub use rank::Rank;
pub use select::{Select0, Select1};

#[derive(Clone, Debug, Default)]
pub struct Summary {
    seq16_nums: usize,
    seq16_ones: u128,
    seq16_size: u128,

    seq64_nums: usize,
    seq64_ones: u128,
    seq64_size: u128,

    rle16_nums: usize,
    rle16_ones: u128,
    rle16_size: u128,

    pub total_nums: usize,
    pub total_ones: u128,
    pub total_size: u128,
}

impl ::std::iter::Sum<vec32::Stats> for Summary {
    fn sum<I>(iter: I) -> Summary
    where
        I: Iterator<Item = vec32::Stats>,
    {
        let mut sum = Summary::default();
        for stat in iter {
            match stat.kind {
                vec32::BlockKind::Seq16 => {
                    sum.seq16_nums += 1;
                    sum.seq16_ones += stat.ones;
                    sum.seq16_size += stat.size;
                }
                vec32::BlockKind::Seq64 => {
                    sum.seq64_nums += 1;
                    sum.seq64_ones += stat.ones;
                    sum.seq64_size += stat.size;
                }
                vec32::BlockKind::Rle16 => {
                    sum.rle16_nums += 1;
                    sum.rle16_ones += stat.ones;
                    sum.rle16_size += stat.size;
                }
            }
        }
        sum.total_nums = sum.seq16_nums + sum.seq64_nums + sum.rle16_nums;
        sum.total_ones = sum.seq16_ones + sum.seq64_ones + sum.rle16_ones;
        sum.total_size = sum.seq16_size + sum.seq64_size + sum.rle16_size;
        sum
    }
}

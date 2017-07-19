#![feature(i128_type)]
#![feature(associated_type_defaults)]
#![feature(conservative_impl_trait)]
#![feature(inclusive_range)]
#![feature(inclusive_range_syntax)]

// #![deny(warnings)]

#[macro_use]
extern crate karabiner;
extern crate itertools;

// Broadword implementation of rank/select queries
// ( http://sux.di.unimi.it/paper.pdf );
// Springer Berlin Heidelberg, 2008. 154-168.

#[macro_use]
mod macros;
mod rank;
mod select;
mod prim;
mod dict;
mod block;
mod map16;
mod map32;
mod map64;

pub mod pair;
pub use prim::UnsignedInt;
pub use rank::Rank;
pub use select::{Select0, Select1};
pub use dict::Dict;
pub use map16::Map16;
pub use map32::Map32;
pub use map64::Map64;

static TRUE: &bool = &true;
static FALSE: &bool = &false;

#[derive(Clone, Debug, Default)]
pub struct Summary {
    seq16_nums: usize,
    seq16_ones: u64,
    seq16_size: u64,

    seq64_nums: usize,
    seq64_ones: u64,
    seq64_size: u64,

    rle16_nums: usize,
    rle16_ones: u64,
    rle16_size: u64,

    pub total_nums: usize,
    pub total_ones: u64,
    pub total_size: u64,
}

impl ::std::iter::Sum<block::Stats> for Summary {
    fn sum<I>(iter: I) -> Summary
    where
        I: Iterator<Item = block::Stats>,
    {
        let mut sum = Summary::default();
        for stat in iter {
            match stat.kind {
                block::Kind::Seq16 => {
                    sum.seq16_nums += 1;
                    sum.seq16_ones += stat.ones;
                    sum.seq16_size += stat.size;
                }
                block::Kind::Seq64 => {
                    sum.seq64_nums += 1;
                    sum.seq64_ones += stat.ones;
                    sum.seq64_size += stat.size;
                }
                block::Kind::Rle16 => {
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

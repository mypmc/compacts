extern crate compacts_prim as prim;
extern crate compacts_bits as bits;

use std::ops::Index;
use prim::UnsignedInt;

pub use bits::{Rank, Select0, Select1};

pub trait Dict<T>: Index<T>
where
    <Self as Index<T>>::Output: PartialEq<Self::Item>,
{
    /// Associated items to this dictionary.
    type Item;

    /// Result type of `rank`.
    type Rank: From<T>;

    fn size(&self) -> Self::Rank;

    /// Returns count of `Item` in `0...i`.
    fn rank(&self, item: &Self::Item, i: T) -> Self::Rank;

    /// Returns the position of the `c+1`-th appearance of `Item`.
    fn select(&self, item: &Self::Item, c: T) -> Option<T>;
}

pub trait BitDict<T: UnsignedInt>
    : Index<T, Output = bool> + Rank<T> + Select0<T> + Select1<T> {
    fn rank0(&self, T) -> Self::Weight;
    fn rank1(&self, T) -> Self::Weight;

    fn select0(&self, T) -> Option<T>;
    fn select1(&self, T) -> Option<T>;
}

impl<T, U> BitDict<T> for U
where
    T: UnsignedInt,
    U: Index<T, Output = bool> + Rank<T> + Select0<T> + Select1<T>,
{
    fn rank0(&self, i: T) -> Self::Weight {
        <Self as Rank<T>>::rank0(self, i)
    }
    fn rank1(&self, i: T) -> Self::Weight {
        <Self as Rank<T>>::rank1(self, i)
    }

    fn select0(&self, c: T) -> Option<T> {
        <Self as Select0<T>>::select0(self, c)
    }
    fn select1(&self, c: T) -> Option<T> {
        <Self as Select1<T>>::select1(self, c)
    }
}

impl<T, U> Dict<T> for U
where
    T: UnsignedInt,
    U: BitDict<T>,
    U::Weight: From<T>,
{
    type Item = bool;
    type Rank = U::Weight;

    fn size(&self) -> Self::Rank {
        <Self as Rank<T>>::SIZE
    }

    fn rank(&self, item: &Self::Item, i: T) -> Self::Rank {
        if *item {
            BitDict::rank1(self, i)
        } else {
            BitDict::rank0(self, i)
        }
    }

    fn select(&self, item: &Self::Item, c: T) -> Option<T> {
        if *item {
            BitDict::select1(self, c)
        } else {
            BitDict::select0(self, c)
        }
    }
}

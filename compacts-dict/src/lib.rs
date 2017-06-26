extern crate compacts_prim as prim;
extern crate compacts_bits as bits;

use std::ops::Index;
use prim::UnsignedInt;

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
    : Index<T, Output = bool> + bits::Rank<T> + bits::Select0<T> + bits::Select1<T>
    {
}

impl<T, U> BitDict<T> for U
where
    T: UnsignedInt,
    U: Index<T, Output = bool> + bits::Rank<T> + bits::Select0<T> + bits::Select1<T>,
{
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
        <Self as bits::Rank<T>>::SIZE
    }

    fn rank(&self, item: &Self::Item, i: T) -> Self::Rank {
        if *item {
            self.rank1(i)
        } else {
            self.rank0(i)
        }
    }

    fn select(&self, item: &Self::Item, c: T) -> Option<T> {
        if *item {
            self.select1(c)
        } else {
            self.select0(c)
        }
    }
}

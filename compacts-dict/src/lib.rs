extern crate compacts_prim as prim;
extern crate compacts_bits as bits;

use std::ops::Index;
use prim::{UnsignedInt, Zero, Cast};

pub trait Dict<T: UnsignedInt>: Index<T>
where
    <Self as Index<T>>::Output: PartialEq<Self::Item>,
{
    /// Associated items to this dictionary.
    type Item;

    /// Result type of `rank`.
    type Rank: UnsignedInt + From<T>;

    fn size(&self) -> Self::Rank;

    /// Returns count of `Item` in `0...i`.
    fn rank(&self, item: &Self::Item, i: T) -> Self::Rank {
        let mut r = <Self::Rank as Zero>::zero();
        let mut j = <T as Zero>::zero();
        while j <= i {
            if &self[j] == item {
                r.incr();
            }
            j.incr();
        }
        r
    }

    /// Returns the position of the `c+1`-th appearance of `Item`, by binary search.
    fn select(&self, item: &Self::Item, c: T) -> Option<T> {
        let zero = <Self::Rank as Zero>::zero();
        let size = self.size();
        let pos = prim::search(&(zero..size), |i| {
            Cast::from::<Self::Rank>(i)
                .and_then(|conv: T| {
                    let rank = self.rank(item, conv);
                    Cast::from::<Self::Rank>(rank)
                })
                .map_or(false, |rank: T| rank > c)
        });
        if pos < size {
            Some(
                Cast::from::<Self::Rank>(pos).expect("if pos < size, cast must not failed"),
            )
        } else {
            None
        }
    }
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
        <Self as bits::Rank<T>>::size(self)
    }

    fn rank(&self, item: &Self::Item, i: T) -> Self::Rank {
        if *item { self.rank1(i) } else { self.rank0(i) }
    }

    // to test default select implementation.
    #[cfg(not(test))]
    fn select(&self, item: &Self::Item, c: T) -> Option<T> {
        if *item {
            self.select1(c)
        } else {
            self.select0(c)
        }
    }
}

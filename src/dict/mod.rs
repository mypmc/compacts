use std::ops::{Index, Range};

pub mod prim;
use self::prim::{Uint, Cast};

mod ranked;
mod select;
pub use self::ranked::Ranked;
pub use self::select::{Select0, Select1};

pub trait Dict<T: Uint>: Index<T>
    where <Self as Index<T>>::Output: PartialEq<Self::Item>
{
    /// Associated items to this dictionary.
    type Item;

    /// Result type of `rank`.
    type Rank: Uint;

    fn size(&self) -> Self::Rank;

    /// Returns count of `Item` in `0...i`.
    fn rank(&self, item: &Self::Item, i: T) -> Self::Rank {
        let mut r = <Self::Rank as prim::Zero>::zero();
        let mut j = <T as prim::Zero>::zero();
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
        let zero = <Self::Rank as prim::Zero>::zero();
        let size = self.size();
        let pos = search(&(zero..size), |i| {
            Cast::from::<Self::Rank>(i)
                .and_then(|conv: T| {
                              let rank = self.rank(item, conv);
                              Cast::from::<Self::Rank>(rank)
                          })
                .map_or(false, |rank: T| rank > c)
        });
        if pos < size {
            Some(Cast::from::<Self::Rank>(pos).expect("if pos < size, cast must not failed"))
        } else {
            None
        }
    }
}

pub trait BitDict<T: Uint>
    : Index<T, Output = bool> + Ranked<T> + Select0<T> + Select1<T> {
}

impl<T, U> BitDict<T> for U
    where T: Uint,
          U: Index<T, Output = bool> + Ranked<T> + Select0<T> + Select1<T>
{
}

impl<T, U> Dict<T> for U
    where T: Uint,
          U: BitDict<T>
{
    type Item = bool;
    type Rank = U::Weight;

    fn size(&self) -> Self::Rank {
        <Self as Ranked<T>>::size(self)
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

/// Find the smallest index i in range at which f(i) is true, assuming that
/// f(i) == true implies f(i+1) == true.
pub fn search<T, F>(range: &Range<T>, f: F) -> T
    where T: Uint,
          F: Fn(T) -> bool
{
    let two = T::from(2).unwrap();
    let mut i = range.start;
    let mut j = range.end;
    while i < j {
        let h = i + (j - i) / two;
        if f(h) {
            j = h; // f(j) == true
        } else {
            i = h.succ(); // f(i-1) == false
        }
    }
    i
}

/// Bits is a struct to implement Index for u64.
pub struct Bits(pub u64);
impl Bits {
    pub fn new(u: u64) -> Self {
        Bits(u)
    }
}

macro_rules! impl_Index_for_Bits {
    ( $( $index:ty ),* ) => ($(
        impl Index<$index> for Bits {
            type Output = bool;
            fn index(&self, i: $index) -> &Self::Output {
                // debug_assert!(i < 64); // should be panic?
                if i >= 64 { return prim::FALSE; }
                let u = self.0;
                if u & (1 << i) != 0 { prim::TRUE } else { prim::FALSE }
            }
        }
    )*)
}
impl_Index_for_Bits!(usize, u64, u32, u16, u8);

use std::ops::{Range, RangeBounds};

use super::{cast, from_any_bounds, Uint, OUT_OF_BOUNDS};

/// `FiniteBits` denotes types with a finite, fixed number of bits.
///
/// This trait is for types intended to use as a component of the bits container.
/// e.g.) T of `Map<T>`, V of `EntryMap<K, V>`
pub trait FiniteBits: Clone + Count {
    /// The potential bit size.
    ///
    /// This constant value corresponds to total of enabled/disabled bits.
    const BITS: u64;

    /// Returns an empty bits container.
    ///
    /// The number of disabled bits of an empty instance must be equal to `BITS`.
    fn empty() -> Self;
}

/// `Count` is a trait that counts the number of enabled/disabled bits in the container.
///
/// Every method have a cycled default implementations.
/// At least two methods need be re-defined.
pub trait Count {
    /// The value corresponds to total of enabled/disabled bits.
    /// Defined as `count1 + count0`.
    fn bits(&self) -> u64 {
        self.count1() + self.count0()
    }

    /// Return the number of enabled bits in the container.
    /// Defined as `bits - count0`.
    ///
    /// Counting bits is not always `O(1)`. It depends on the implementation.
    fn count1(&self) -> u64 {
        self.bits() - self.count0()
    }

    /// Return the number of disabled bits in the container.
    /// Defined as `bits - count1`.
    ///
    /// Counting bits is not always `O(1)`. It depends on the implementation.
    fn count0(&self) -> u64 {
        self.bits() - self.count1()
    }
}

/// `Access` is a trait to test bit.
pub trait Access {
    fn access(&self, index: u64) -> bool;

    /// Return the positions of all enabled bits in the container.
    ///
    /// Default implementation is just a accessing to all bits.
    ///
    /// ```
    /// use compacts::bit::ops::Access;
    /// let word = [0b_10101010_u8, 0b_11110000_u8];
    /// let bits = word.iterate().collect::<Vec<_>>();
    /// assert_eq!(bits, vec![1, 3, 5, 7, 12, 13, 14, 15]);
    /// ```
    fn iterate<'a>(&'a self) -> Box<dyn Iterator<Item = u64> + 'a>
    where
        Self: Count,
    {
        Box::new((0..self.bits()).filter(move |&i| self.access(i)))
    }
}

#[doc(hidden)]
pub enum Excess {
    Rank1(u64), // rank1 > rank0
    Rank0(u64), // rank1 < rank0
}

/// Search the smallest index in range at which f(i) is true,
/// assuming that f(i) == true implies f(i+1) == true.
fn search_index<T: Uint>(k: T, func: impl Fn(T) -> bool) -> T {
    let mut i = 0;
    let mut j = cast::<T, usize>(k);
    while i < j {
        let h = i + (j - i) / 2;
        if func(cast(h)) {
            j = h; // f(j) == true
        } else {
            i = h + 1; // f(i-1) == false
        }
    }
    cast(i) // f(i-1) == false && f(i) (= f(j)) == true
}

/// `Rank` is a generization of `Count`.
///
/// Both `rank1` and `rank0` have default implementation, but these are cycled.
/// Either `rank1` or `rank0` need to be re-defined.
pub trait Rank: Count {
    /// Returns the number of enabled bits in `[0, i)`.
    /// Defined as `i - rank0`.
    ///
    /// `rank1(self.bits())` is equal to `count1()`.
    ///
    /// # Panics
    ///
    /// Panics if `i > bits`.
    fn rank1(&self, i: u64) -> u64 {
        assert!(i <= self.bits(), OUT_OF_BOUNDS);
        i - self.rank0(i)
    }

    /// Returns the number of disabled bits in `[0, i)`.
    /// Difined as `i - rank1`.
    ///
    /// `rank0(self.bits())` is equal to `count0()`.
    ///
    /// # Panics
    ///
    /// Panics if `i > bits`.
    fn rank0(&self, i: u64) -> u64 {
        assert!(i <= self.bits(), OUT_OF_BOUNDS);
        i - self.rank1(i)
    }

    /// Searches the position of `n+1`th enabled bit by binary search.
    #[doc(hidden)]
    fn search1(&self, n: u64) -> Option<u64> {
        if n < self.count1() {
            Some(search_index(self.bits(), |k| self.rank1(k) > n) - 1)
        } else {
            None
        }
    }

    /// Searches the position of `n+1`th disabled bit by binary search.
    #[doc(hidden)]
    fn search0(&self, n: u64) -> Option<u64> {
        if n < self.count0() {
            Some(search_index(self.bits(), |k| self.rank0(k) > n) - 1)
        } else {
            None
        }
    }

    /// Returns an excess of rank.
    #[doc(hidden)]
    fn excess(&self, i: u64) -> Option<Excess> {
        use std::cmp::Ordering::{Equal as EQ, Greater as GE, Less as LE};

        let rank1 = self.rank1(i);
        let rank0 = i - rank1;
        match rank1.cmp(&rank0) {
            EQ => None,
            LE => Some(Excess::Rank0(rank0 - rank1)),
            GE => Some(Excess::Rank1(rank1 - rank0)),
        }
    }
}

/// Right inverse of `rank1`.
pub trait Select1: Count {
    /// Returns the position of 'n+1'th occurences of `1`.
    fn select1(&self, n: u64) -> Option<u64>;
}

/// Right inverse of `rank0`.
pub trait Select0: Count {
    /// Returns the position of 'n+1'th occurences of `0`.
    fn select0(&self, n: u64) -> Option<u64>;
}

/// `Assign` is a trait to enable/disable bits.
pub trait Assign<Idx> {
    type Output;
    fn set1(&mut self, index: Idx) -> Self::Output;
    fn set0(&mut self, index: Idx) -> Self::Output;
}

impl<'a, T: ?Sized + Count + Assign<U>, U: Clone> Assign<&'a U> for T {
    type Output = <T as Assign<U>>::Output;
    fn set1(&mut self, r: &'a U) -> Self::Output {
        self.set1(r.clone())
    }
    fn set0(&mut self, r: &'a U) -> Self::Output {
        self.set0(r.clone())
    }
}

macro_rules! implsRangeBoundsAssign {
    ($($Type:ty),*) => ($(
        impl<T: ?Sized + Count + Assign<Range<u64>>> Assign<$Type> for T {
            type Output = <T as Assign<Range<u64>>>::Output;
            fn set1(&mut self, r: $Type) -> Self::Output {
                self.set1(from_any_bounds(&r, self.bits()))
            }
            fn set0(&mut self, r: $Type) -> Self::Output {
                self.set0(from_any_bounds(&r, self.bits()))
            }
        }
    )*)
}
implsRangeBoundsAssign!(
    std::ops::RangeTo<u64>,
    std::ops::RangeFull,
    std::ops::RangeFrom<u64>,
    std::ops::RangeInclusive<u64>,
    std::ops::RangeToInclusive<u64>
);

/// `Read` is a trait to read a word from the bits container.
pub trait Read<W: Uint> {
    fn read<Idx: RangeBounds<u64>>(&self, i: Idx) -> W;
}

// impl<'a, T, W, Idx> Read<W, &'a Idx> for T
// where
//     T: ?Sized + Read<W, Idx>,
//     W: Uint,
//     Idx: Clone,
// {
//     fn read(&self, i: &'a Idx) -> W {
//         self.read(i.clone())
//     }
// }

// macro_rules! implsRangeBoundsRead {
//     ($($Type:ty),*) => ($(
//         impl<T, W> Read<W, $Type> for T
//         where
//             T: ?Sized + Count + Read<W, Range<u64>>,
//             W: Uint,
//         {
//             fn read(&self, i: $Type) -> W {
//                 self.read(from_bounds(&i, self.bits()))
//             }
//         }
//     )*)
// }
// implsRangeBoundsRead!(
//     std::ops::RangeTo<u64>,
//     std::ops::RangeFull,
//     std::ops::RangeFrom<u64>,
//     std::ops::RangeInclusive<u64>,
//     std::ops::RangeToInclusive<u64>
// );

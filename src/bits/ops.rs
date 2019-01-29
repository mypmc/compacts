use std::ops::{Bound, Range, RangeBounds};

use super::{ucast, UnsignedInt, OUT_OF_BOUNDS};

/// `FiniteBits` denotes types with a finite, fixed number of bits.
///
/// This trait is for types intended to use as a component of the bits container.
/// e.g.) T of `Map<T>`, V of `PageMap<K, V>`
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
    /// use compacts::bits::ops::Access;
    /// let word = [0b_10101010_u8, 0b_11110000_u8];
    /// let bits = word.iterate().collect::<Vec<_>>();
    /// assert_eq!(bits, vec![1, 3, 5, 7, 12, 13, 14, 15]);
    /// ```
    fn iterate<'a>(&'a self) -> Box<dyn Iterator<Item = u64> + 'a>
    where
        Self: Count,
    {
        Box::new((0..self.bits()).filter_map(move |i| if self.access(i) { Some(i) } else { None }))
    }
}

#[doc(hidden)]
pub enum Excess {
    Rank1(u64), // rank1 > rank0
    Rank0(u64), // rank1 < rank0
}

/// Search the smallest index in range at which f(i) is true,
/// assuming that f(i) == true implies f(i+1) == true.
fn search_index<T: UnsignedInt>(k: T, func: impl Fn(T) -> bool) -> T {
    let mut i = 0;
    let mut j = ucast::<T, usize>(k);
    while i < j {
        let h = i + (j - i) / 2;
        if func(ucast(h)) {
            j = h; // f(j) == true
        } else {
            i = h + 1; // f(i-1) == false
        }
    }
    ucast(i) // f(i-1) == false && f(i) (= f(j)) == true
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

#[allow(clippy::range_plus_one)]
#[rustfmt::skip]
pub(crate) fn from_bounds<R: RangeBounds<u64>>(range: &'_ R, bits: u64) -> Range<u64> {
    use Bound::*;
    match (range.start_bound(), range.end_bound()) {

        (Included(&i), Included(&j)) if i   < bits && i <= j && j <  bits => i   .. j+1,
        (Included(&i), Excluded(&j)) if i   < bits && i <= j && j <= bits => i   .. j,
        (Excluded(&i), Included(&j)) if i+1 < bits && i <  j && j <  bits => i+1 .. j+1,
        (Excluded(&i), Excluded(&j)) if i+1 < bits && i <  j && j <= bits => i+1 .. j,

        // i == 0
        (Unbounded, Included(&j)) if j <  bits => 0 .. j+1,
        (Unbounded, Excluded(&j)) if j <= bits => 0 .. j,

        // j == bits
        (Included(&i), Unbounded) if i   < bits => i   .. bits,
        (Excluded(&i), Unbounded) if i+1 < bits => i+1 .. bits,

        (Unbounded, Unbounded) => 0 .. bits,

        _ => panic!("unexpected range"),
    }
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

macro_rules! implsRangeBounds {
    ($($Type:ty),*) => ($(
        impl<T: ?Sized + Count + Assign<Range<u64>>> Assign<$Type> for T {
            type Output = <T as Assign<Range<u64>>>::Output;
            fn set1(&mut self, r: $Type) -> Self::Output {
                self.set1(from_bounds(&r, self.bits()))
            }
            fn set0(&mut self, r: $Type) -> Self::Output {
                self.set0(from_bounds(&r, self.bits()))
            }
            // fn flip(&mut self, r: $Type) -> Self::Output {
            //     self.flip(from_bounds(&r, self.bits()))
            // }
        }
    )*)
}
implsRangeBounds!(
    std::ops::RangeTo<u64>,
    std::ops::RangeFull,
    std::ops::RangeFrom<u64>,
    std::ops::RangeInclusive<u64>,
    std::ops::RangeToInclusive<u64>
);

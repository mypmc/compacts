//! Defines traits to pretend `[bool]`.
//!
//! Both an index and a weight of bits are represented by `usize`.

use std::ops::{
    Bound, Range, RangeBounds, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive,
};

// pub(crate) use std::convert::{TryFrom, TryInto};

pub(crate) mod private;
pub(crate) use private::Sealed;

use crate::{
    bits::to_exclusive,
    num::{binary_search, Word},
};

/// `Text` is a sequence of `Code`. Typically, `Code` is an unsigned integer.
pub trait Text {
    /// A content of this text.
    type Code;

    /// The size of this text.
    fn size(&self) -> usize;

    /// Counts the occurences of `e` in this text.
    fn count(&self, e: &Self::Code) -> usize;
}

/// `Code`
pub trait Code: Copy + Bits {
    /// DEPTH
    const DEPTH: usize;
    /// MIN
    const MIN: Self;
    /// MAX
    const MAX: Self;
}

/// `Rank` generalizes `Text::count`.
pub trait Rank<Idx = usize>: Text {
    /// Returns the number of occurrence of `code`.
    fn rank(&self, code: &Self::Code, i: Idx) -> usize;
}

/// Selects `n`th item.
pub trait Select: Text {
    /// Selects `n`th code.
    fn select(&self, code: &Self::Code, n: usize) -> Option<usize>;
}

/// The immutable bit sequences.
///
/// ## Defaults
///
/// This trait has methods with circular defaults, so implementator need to redefine methods below.
///
/// ```text
///     - Either count1 or count0
///     - Either rank1  or rank0
/// ```
pub trait Bits {
    /// The size of this bit sequence. The size is always equal to `count1() + count0()`.
    fn size(&self) -> usize;

    /// Reads bit at `i`.
    fn bit(&self, i: usize) -> bool;

    /// Reads `n` bits in `[i, i+n)`, and returns them as the lowest `n` bit of `T`.
    #[doc(hidden)]
    fn getn<T: Word>(&self, i: usize, n: usize) -> T {
        let mut word = T::NONE;
        for b in i..i + n {
            if self.bit(b) {
                word.put1(b - i);
            }
        }
        word
    }

    /// Returns true if all bits are enabled.
    #[inline]
    fn all(&self) -> bool {
        self.count0() == 0
    }

    /// Returns true if any bits are enabled.
    #[inline]
    fn any(&self) -> bool {
        self.count1() > 0
    }

    /// Returns the number of `bit`.
    #[inline]
    fn count(&self, bit: bool) -> usize {
        if bit {
            self.count1()
        } else {
            self.count0()
        }
    }

    /// Counts the occurences of `1`.
    #[inline]
    fn count1(&self) -> usize {
        self.size() - self.count0()
    }

    /// Counts the occurences of `0`.
    #[inline]
    fn count0(&self) -> usize {
        self.size() - self.count1()
    }

    #[inline]
    fn rank<R: RangeBounds<usize>>(&self, bit: bool, range: R) -> usize {
        if bit {
            self.rank1(range)
        } else {
            self.rank0(range)
        }
    }

    #[inline]
    fn rank1<R: RangeBounds<usize>>(&self, range: R) -> usize {
        let (i, j) = to_exclusive(&range, self.size()).expect("out of bounds");
        (j - i) - self.rank0(range)
    }

    #[inline]
    fn rank0<R: RangeBounds<usize>>(&self, range: R) -> usize {
        let (i, j) = to_exclusive(&range, self.size()).expect("out of bounds");
        (j - i) - self.rank1(range)
    }

    #[doc(hidden)]
    #[inline]
    fn search1(&self, n: usize) -> Option<usize> {
        if n < self.count1() {
            Some(binary_search(0, self.size(), |k| self.rank1(..k) > n) - 1)
        } else {
            None
        }
    }

    #[doc(hidden)]
    #[inline]
    fn search0(&self, n: usize) -> Option<usize> {
        if n < self.count0() {
            Some(binary_search(0, self.size(), |k| self.rank0(..k) > n) - 1)
        } else {
            None
        }
    }

    #[inline]
    fn select(&self, bit: bool, n: usize) -> Option<usize> {
        if bit {
            self.select1(n)
        } else {
            self.select0(n)
        }
    }

    /// Returns the position of nth occurrence of `1`.
    #[inline]
    fn select1(&self, n: usize) -> Option<usize> {
        self.search1(n)
    }

    /// Returns the position of nth occurrence of `0`.
    #[inline]
    fn select0(&self, n: usize) -> Option<usize> {
        self.search0(n)
    }

    #[doc(hidden)]
    #[inline]
    fn select1_from(&self, i: usize, n: usize) -> Option<usize> {
        self.select1(self.rank1(..i) + n).map(|pos| pos - i)
    }

    #[doc(hidden)]
    #[inline]
    fn select0_from(&self, i: usize, n: usize) -> Option<usize> {
        self.select0(self.rank0(..i) + n).map(|pos| pos - i)
    }
}

/// `FixedBits` is a fixed size, mutable `Bits`.
pub trait FixedBits: Clone + Bits + BitsMut {
    /// A constant size in bits. This value should be always equal to `size()`.
    const SIZE: usize;

    /// Returns an empty instance.
    fn none() -> Self;
}

/// The mutable bit sequence.
pub trait BitsMut: Bits {
    /// Manipulates bit at `i`.
    #[inline]
    fn put(&mut self, i: usize, bit: bool) {
        if bit {
            self.put1(i)
        } else {
            self.put0(i)
        }
    }

    /// Enables the bit at `i`.
    fn put1(&mut self, i: usize);

    /// Disables the bit at `i`.
    fn put0(&mut self, i: usize);

    /// Flips the bit at `i`.
    #[inline]
    fn flip(&mut self, i: usize) {
        self.put(i, !self.bit(i))
    }

    // Writes `n` bits in `[i, i+n)`.
    // #[doc(hidden)]
    // fn putn<T: Word>(&mut self, i: usize, n: usize, w: T) {
    //     for b in i..i + n {
    //         if w.bit(b - i) {
    //             self.put1(b);
    //         }
    //     }
    // }
}

mod others {
    use super::*;

    impl<T: FixedBits> FixedBits for Box<T> {
        const SIZE: usize = T::SIZE;
        #[inline]
        fn none() -> Self {
            Box::new(T::none())
        }
    }

    impl<T: FixedBits> FixedBits for Option<T> {
        const SIZE: usize = T::SIZE;
        #[inline]
        fn none() -> Self {
            None
        }
    }

    impl<T: ?Sized + Bits> Bits for Box<T> {
        #[inline]
        fn size(&self) -> usize {
            self.as_ref().size()
        }

        #[inline]
        fn bit(&self, i: usize) -> bool {
            self.as_ref().bit(i)
        }

        #[inline]
        fn count1(&self) -> usize {
            self.as_ref().count1()
        }
        #[inline]
        fn count0(&self) -> usize {
            self.as_ref().count0()
        }

        #[inline]
        fn rank1<R: RangeBounds<usize>>(&self, range: R) -> usize {
            self.as_ref().rank1(range)
        }
        #[inline]
        fn rank0<R: RangeBounds<usize>>(&self, range: R) -> usize {
            self.as_ref().rank0(range)
        }

        #[inline]
        fn select1(&self, n: usize) -> Option<usize> {
            self.as_ref().select1(n)
        }
        #[inline]
        fn select0(&self, n: usize) -> Option<usize> {
            self.as_ref().select0(n)
        }

        #[inline]
        fn all(&self) -> bool {
            self.as_ref().all()
        }
        #[inline]
        fn any(&self) -> bool {
            self.as_ref().any()
        }

        #[inline]
        fn getn<U: Word>(&self, i: usize, n: usize) -> U {
            self.as_ref().getn(i, n)
        }
    }

    impl<T: FixedBits> Bits for Option<T> {
        #[inline]
        fn size(&self) -> usize {
            Self::SIZE
        }

        #[inline]
        fn bit(&self, i: usize) -> bool {
            BOUNDS_CHECK!(i < self.size());
            self.as_ref().map_or(false, |t| t.bit(i))
        }

        #[inline]
        fn count1(&self) -> usize {
            self.as_ref().map_or(0, Bits::count1)
        }
        #[inline]
        fn count0(&self) -> usize {
            self.as_ref().map_or(Self::SIZE, Bits::count0)
        }

        #[inline]
        fn rank1<R: RangeBounds<usize>>(&self, i: R) -> usize {
            self.as_ref().map_or(0, |t| t.rank1(i))
        }
        #[inline]
        fn rank0<R: RangeBounds<usize>>(&self, i: R) -> usize {
            self.size() - self.rank1(i)
        }

        #[inline]
        fn all(&self) -> bool {
            self.as_ref().map_or(true, Bits::all)
        }
        #[inline]
        fn any(&self) -> bool {
            self.as_ref().map_or(false, Bits::any)
        }

        #[inline]
        fn select1(&self, n: usize) -> Option<usize> {
            self.as_ref().and_then(|t| t.select1(n))
        }
        #[inline]
        fn select0(&self, n: usize) -> Option<usize> {
            self.as_ref().map_or(Some(n), |t| t.select0(n))
        }

        #[inline]
        fn getn<W: Word>(&self, i: usize, n: usize) -> W {
            BOUNDS_CHECK!(n <= W::SIZE && i < self.size() && i + n <= self.size());
            self.as_ref().map_or(W::NONE, |t| t.getn(i, n))
        }
    }

    impl<T: ?Sized + BitsMut> BitsMut for Box<T> {
        #[inline]
        fn put1(&mut self, i: usize) {
            self.as_mut().put1(i);
        }

        #[inline]
        fn put0(&mut self, i: usize) {
            self.as_mut().put0(i);
        }

        #[inline]
        fn flip(&mut self, i: usize) {
            self.as_mut().flip(i);
        }

        // #[inline]
        // fn putn<W: Word>(&mut self, i: usize, n: usize, w: W) {
        //     self.as_mut().putn(i, n, w)
        // }
    }

    impl<T: FixedBits> BitsMut for Option<T> {
        fn put1(&mut self, i: usize) {
            BOUNDS_CHECK!(i < self.size());
            self.get_or_insert_with(T::none).put1(i);
        }

        #[inline]
        fn put0(&mut self, i: usize) {
            BOUNDS_CHECK!(i < self.size());
            if let Some(t) = self.as_mut() {
                t.put0(i);
            }
        }

        #[inline]
        fn flip(&mut self, i: usize) {
            BOUNDS_CHECK!(i < self.size());
            self.get_or_insert_with(T::none).flip(i);
        }

        // fn putn<W: Word>(&mut self, i: usize, n: usize, w: W) {
        //     BOUNDS_CHECK!(n <= W::SIZE && i < self.size() && i + n <= self.size());
        //     self.get_or_insert_with(T::none).putn(i, n, w);
        // }
    }
}

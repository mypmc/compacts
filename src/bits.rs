//! `bits`

macro_rules! divrem {
    ($i:expr, $n:expr) => {{
        let x = $i;
        let y = $n;
        (x / y, x % y)
    }};
}

mod mask;
mod rrr;

pub mod pop_vec;
pub use pop_vec::Pop;

pub mod bit_array;
pub mod bit_vec;
pub mod map;
pub use {bit_array::BitArray, bit_vec::BitVec};

pub use {
    mask::{and, and_not, or, xor},
    rrr::Rrr,
};

pub use {
    mask::{And, AndNot, Or, Xor},
    mask::{Difference, Intersection, SymmetricDifference, Union},
    mask::{Fold, Mask},
};

use std::ops::{Bound, RangeBounds};

use crate::{
    num::{Int, Word},
    ops::{private::Sealed, Bits, BitsMut, FixedBits},
};

/// Computes the minimum length of the sequence to store `n` bits.
#[inline]
pub fn blocks<T: FixedBits>(n: usize) -> usize {
    blocks_by(n, T::SIZE)
}

/// Computes the minimum length of the sequence to store `n` bits.
#[inline]
pub const fn blocks_by(n: usize, block_size: usize) -> usize {
    // If we want 17 bits, dividing by 32 will produce 0. So we add 1 to make sure we reserve enough.
    // But if we want exactly a multiple of `block_size`, this will actually allocate one too many.
    n / block_size + (n % block_size > 0) as usize
}

/// Allocates an empty bitvector with the specified bit size.
pub fn sized<T: FixedBits>(n: usize) -> Vec<T> {
    sized_with(n, T::none)
}

/// Returns an empty `Vec` with the at least specified capacity in bits.
#[inline]
pub fn with_capacity<T: FixedBits>(n: usize) -> Vec<T> {
    Vec::with_capacity(blocks::<T>(n))
}

pub(crate) fn sized_with<T, F>(bits: usize, mut f: F) -> Vec<T>
where
    T: FixedBits,
    F: FnMut() -> T,
{
    std::iter::from_fn(|| Some(f()))
        .take(blocks_by(bits, T::SIZE))
        .collect()
}

pub(crate) fn to_exclusive<R: RangeBounds<usize>>(range: &R, max: usize) -> Option<(usize, usize)> {
    let start = match range.start_bound() {
        Bound::Included(&n) => n,
        Bound::Excluded(&n) => n + 1,
        Bound::Unbounded => 0,
    };
    let end = match range.end_bound() {
        Bound::Included(&n) => n + 1,
        Bound::Excluded(&n) => n,
        Bound::Unbounded => max,
    };

    if start <= end && end <= max {
        Some((start, end))
    } else {
        None
    }
}

impl<T: FixedBits> Bits for [T] {
    /// ```
    /// # use compacts::ops::Bits;
    /// let v: &[u64] = &[0b10101100000, 0b0000100000];
    /// let w: &[u64] = &[];
    /// assert_eq!(v.size(), 128);
    /// assert_eq!(w.size(), 0);
    /// ```
    #[inline]
    fn size(&self) -> usize {
        self.len() * T::SIZE // blocks length * bits of block
    }

    /// ```
    /// # use compacts::ops::Bits;
    /// let v: &[u64] = &[0b_00000101u64, 0b01100011, 0b01100000];
    /// assert!( v.bit(0));
    /// assert!(!v.bit(1));
    /// assert!( v.bit(2));
    /// assert!(!v.bit(3));
    ///
    /// let w = &v[1..]; // slicing is performed blockwise.
    /// assert!( w.bit(0));
    /// assert!( w.bit(1));
    /// assert!(!w.bit(2));
    /// ```
    #[inline]
    fn bit(&self, i: usize) -> bool {
        assert!(i < self.size());
        let (i, o) = divrem!(i, T::SIZE);
        self[i].bit(o)
    }

    fn getn<W: Word>(&self, i: usize, n: usize) -> W {
        let mut cur = 0;
        let mut out = W::NONE;
        slice::ranges(self, i, i + n, |t, b1, b2| {
            let len = b2 - b1;
            out |= t.getn::<W>(b1, len) << cur;
            cur += len;
        });
        debug_assert_eq!(cur, n);
        out
    }

    /// ```
    /// # use compacts::ops::Bits;
    /// let v: &[u64] = &[0b10101100000, 0b0000100000];
    /// assert_eq!(v.count1(), 5);
    /// assert_eq!(v.count0(), 123);
    /// ```
    #[inline]
    fn count1(&self) -> usize {
        self.iter().fold(0, |acc, block| acc + block.count1())
    }

    /// ```
    /// # use compacts::ops::Bits;
    /// let v: &[u64] = &[0b10101100000, 0b0000100000];
    /// assert_eq!(v.count1(), 5);
    /// assert_eq!(v.count0(), 123);
    /// ```
    #[inline]
    fn count0(&self) -> usize {
        self.iter().fold(0, |acc, block| acc + block.count0())
    }

    /// ```
    /// # use compacts::ops::Bits;
    /// let v: &[u64] = &[ 0,  0,  0];
    /// let w: &[u64] = &[!0, !0, !0];
    /// assert!(!v.all());
    /// assert!( w.all());
    /// ```
    #[inline]
    fn all(&self) -> bool {
        self.iter().all(Bits::all)
    }

    /// ```
    /// # use compacts::ops::Bits;
    /// let v: &[u64] = &[ 0,  0,  0];
    /// let w: &[u64] = &[!0, !0, !0];
    /// assert!(!v.any());
    /// assert!( w.any());
    /// ```
    #[inline]
    fn any(&self) -> bool {
        self.iter().any(Bits::any)
    }

    #[inline]
    fn rank1<R: RangeBounds<usize>>(&self, range: R) -> usize {
        match to_exclusive(&range, self.size()).expect("out of bounds") {
            (0, j) => {
                let (q, r) = divrem!(j, T::SIZE);
                self[..q].count1() + self.get(q).map_or(0, |p| p.rank1(0..r))
            }
            (i, j) => {
                let (q0, r0) = divrem!(i, T::SIZE);
                let (q1, r1) = divrem!(j, T::SIZE);
                if q0 == q1 {
                    self[q0].rank1(r0..r1)
                } else {
                    self[q0].rank1(r0..)
                        + self[q0 + 1..q1].count1()
                        + self.get(q1).map_or(0, |b| b.rank1(..r1))
                }
            }
        }
    }

    #[inline]
    fn select1(&self, mut nth: usize) -> Option<usize> {
        for (i, v) in self.iter().enumerate() {
            let count1 = v.count1();
            if nth < count1 {
                return Some(i * T::SIZE + v.select1(nth).expect("remain < count"));
            }
            nth -= count1;
        }
        None
    }

    #[inline]
    fn select0(&self, mut nth: usize) -> Option<usize> {
        for (i, v) in self.iter().enumerate() {
            let count0 = v.count0();
            if nth < count0 {
                return Some(i * T::SIZE + v.select0(nth).expect("remain < count"));
            }
            nth -= count0;
        }
        None
    }
}

impl<T: FixedBits> BitsMut for [T] {
    /// Enables bit at `index`.
    #[inline]
    fn put1(&mut self, i: usize) {
        assert!(i < self.size());
        let (i, o) = divrem!(i, T::SIZE);
        self[i].put1(o);
    }

    /// Disables bit at `index`.
    #[inline]
    fn put0(&mut self, i: usize) {
        assert!(i < self.size());
        let (i, o) = divrem!(i, T::SIZE);
        self[i].put0(o);
    }

    /// Flip bit at `index`.
    #[inline]
    fn flip(&mut self, i: usize) {
        assert!(i < self.size());
        let (i, o) = divrem!(i, T::SIZE);
        self[i].flip(o);
    }

    // fn putn<W: Word>(&mut self, i: usize, n: usize, num: W) {
    //     // assert!(n <= W::BITS && i < self.size() && i + n <= self.size());
    //     let mut cur = 0;
    //     ranges_mut(self, i, i + n, |t, b1, b2| {
    //         let len = b2 - b1;
    //         t.putn::<W>(b1, len, num.getn(cur, len));
    //         cur += len;
    //     });
    // }
}

impl<T: Word, A: ?Sized + AsRef<[T]>> Intersection<A> for [T] {
    fn intersection(&mut self, slice: &A) {
        let slice = slice.as_ref();
        assert_eq!(self.len(), slice.len());
        for (v1, &v2) in self.iter_mut().zip(slice) {
            *v1 &= v2;
        }
    }
}

impl<T: Word, A: ?Sized + AsRef<[T]>> Union<A> for [T] {
    fn union(&mut self, slice: &A) {
        let slice = slice.as_ref();
        assert_eq!(self.len(), slice.len());
        for (v1, &v2) in self.iter_mut().zip(slice) {
            *v1 |= v2;
        }
    }
}

impl<T: Word, A: ?Sized + AsRef<[T]>> Difference<A> for [T] {
    fn difference(&mut self, slice: &A) {
        let slice = slice.as_ref();
        assert_eq!(self.len(), slice.len());
        for (v1, &v2) in self.iter_mut().zip(slice) {
            *v1 &= !v2;
        }
    }
}

impl<T: Word, A: ?Sized + AsRef<[T]>> SymmetricDifference<A> for [T] {
    fn symmetric_difference(&mut self, slice: &A) {
        let slice = slice.as_ref();
        assert_eq!(self.len(), slice.len());
        for (v1, &v2) in self.iter_mut().zip(slice) {
            *v1 ^= v2;
        }
    }
}

impl<T: FixedBits> Bits for Vec<T> {
    #[inline]
    fn size(&self) -> usize {
        self.as_slice().size()
    }

    /// ```
    /// # use compacts::ops::Bits;
    /// let v = vec![0b00000101u64, 0b01100011, 0b01100000];
    /// let w = &v[1..];
    /// assert_eq!(v.size(), 192);
    /// assert_eq!(w.size(), 128);
    /// assert_eq!(v.count1(), 8);
    /// assert_eq!(w.count1(), 6);
    ///
    /// assert!( v.bit(0));
    /// assert!(!v.bit(1));
    /// assert!( v.bit(2));
    /// assert!(!v.bit(3));
    ///
    /// assert!( w.bit(0));
    /// assert!( w.bit(1));
    /// assert!(!w.bit(2));
    /// ```
    #[inline]
    fn bit(&self, i: usize) -> bool {
        self.as_slice().bit(i)
    }

    #[inline]
    fn getn<W: Word>(&self, i: usize, n: usize) -> W {
        self.as_slice().getn(i, n)
    }

    #[inline]
    fn count1(&self) -> usize {
        self.as_slice().count1()
    }
    #[inline]
    fn count0(&self) -> usize {
        self.as_slice().count0()
    }

    #[inline]
    fn rank1<R: RangeBounds<usize>>(&self, range: R) -> usize {
        self.as_slice().rank1(range)
    }
    #[inline]
    fn rank0<R: RangeBounds<usize>>(&self, range: R) -> usize {
        self.as_slice().rank0(range)
    }

    #[inline]
    fn select1(&self, n: usize) -> Option<usize> {
        self.as_slice().select1(n)
    }
    #[inline]
    fn select0(&self, n: usize) -> Option<usize> {
        self.as_slice().select0(n)
    }

    #[inline]
    fn all(&self) -> bool {
        self.as_slice().all()
    }
    #[inline]
    fn any(&self) -> bool {
        self.as_slice().any()
    }
}

impl<T: FixedBits> BitsMut for Vec<T> {
    #[inline]
    fn put1(&mut self, i: usize) {
        self.as_mut_slice().put1(i);
    }
    #[inline]
    fn put0(&mut self, i: usize) {
        self.as_mut_slice().put0(i);
    }
    #[inline]
    fn flip(&mut self, i: usize) {
        self.as_mut_slice().flip(i);
    }

    // #[inline]
    // fn putn<W: Word>(&mut self, i: usize, n: usize, w: W) {
    //     self.as_mut_slice().putn(i, n, w);
    // }
}

impl<T: Word, U: ?Sized> Intersection<U> for Vec<T>
where
    [T]: Intersection<U>,
{
    #[inline]
    fn intersection(&mut self, slice: &U) {
        self.as_mut_slice().intersection(slice);
    }
}

impl<T: Word, U: ?Sized> Union<U> for Vec<T>
where
    [T]: Union<U>,
{
    #[inline]
    fn union(&mut self, slice: &U) {
        self.as_mut_slice().union(slice);
    }
}

impl<T: Word, U: ?Sized> Difference<U> for Vec<T>
where
    [T]: Difference<U>,
{
    #[inline]
    fn difference(&mut self, slice: &U) {
        self.as_mut_slice().difference(slice);
    }
}

impl<T: Word, U: ?Sized> SymmetricDifference<U> for Vec<T>
where
    [T]: SymmetricDifference<U>,
{
    #[inline]
    fn symmetric_difference(&mut self, slice: &U) {
        self.as_mut_slice().symmetric_difference(slice);
    }
}

mod slice {
    use super::*;

    pub fn ranges<T: FixedBits, F>(slice: &[T], i: usize, j: usize, mut f: F)
    where
        F: FnMut(&T, usize, usize),
    {
        do_while(slice, i, j, |t, a, b| {
            f(t, a, b);
            true
        });
    }

    // Iterates over `slice` as bit container from `i` to `j`, yielding `(T, start, end)`.
    fn do_while<T, F>(slice: &[T], i: usize, j: usize, mut f: F)
    where
        T: FixedBits,
        F: FnMut(&T, usize, usize) -> bool,
    {
        if i == j {
            return;
        }

        let bits = T::SIZE;
        let (q0, r0) = divrem!(i, bits);
        let (q1, r1) = divrem!(j, bits);

        if q0 == q1 {
            // fit in one block
            if !f(&slice[q0], r0, r1) {
                return;
            }
        } else {
            // spans many blocks
            assert!(q0 < q1);
            let mut remain = j - i;

            if !f(&slice[q0], r0, bits) {
                return;
            }
            remain -= bits - r0;

            for t in &slice[q0 + 1..q1] {
                if !f(t, 0, bits) {
                    return;
                }
            }
            remain -= slice[q0 + 1..q1].len() * bits;

            if remain > 0 && q1 < slice.len() {
                f(&slice[q1], 0, remain);
            }
        }
    }

    // fn ranges_mut<T: FixedBits, F>(slice: &mut [T], i: usize, j: usize, mut f: F)
    // where
    //     F: FnMut(&mut T, usize, usize),
    // {
    //     do_while_mut(slice, i, j, T::SIZE, |t, a, b| {
    //         f(t, a, b);
    //         true
    //     });
    // }

    // fn do_while_mut<T: FixedBits, F>(slice: &mut [T], i: usize, j: usize, bits: usize, mut f: F)
    // where
    //     F: FnMut(&mut T, usize, usize) -> bool,
    // {
    //     if i == j {
    //         return;
    //     }

    //     let (q0, r0) = divrem!(i, bits);
    //     let (q1, r1) = divrem!(j, bits);
    //     if q0 == q1 {
    //         // fit in one block
    //         if !f(&mut slice[q0], r0, r1) {
    //             return;
    //         }
    //     } else {
    //         // spans many blocks
    //         assert!(q0 < q1);
    //         let mut remain = j - i;

    //         if !f(&mut slice[q0], r0, bits) {
    //             return;
    //         }
    //         remain -= bits - r0;

    //         for t in &mut slice[q0 + 1..q1] {
    //             if !f(t, 0, bits) {
    //                 return;
    //             }
    //         }
    //         remain -= slice[q0 + 1..q1].len() * bits;

    //         if remain > 0 && q1 < slice.len() {
    //             f(&mut slice[q1], 0, remain);
    //         }
    //     }
    // }
}

/// `Words` is a fixed size array of word.
pub trait Words: 'static + Copy + FixedBits + Sealed {
    /// An unsigned int, the element of the array.
    type Word: Word;

    /// The length of the array.
    const LEN: usize;

    /// The size in slice of the array.
    #[doc(hidden)]
    const BITS: usize = <Self::Word as Int>::BITS * Self::LEN;

    /// Constructs an empty word array.
    #[inline]
    #[doc(hidden)]
    fn empty() -> Self {
        Self::splat(<Self::Word as Int>::NONE)
    }

    /// Constructs the word array from bit pattern.
    fn splat(word: Self::Word) -> Self; // [Self::Word; Self::LEN]

    /// Constructs a boxed slice of words.
    fn boxed(this: Self) -> Box<[Self::Word]>;

    // should be replaced by std::array::FixedSizeArray;
    #[doc(hidden)]
    fn as_ref_words(&self) -> &[Self::Word];
    #[doc(hidden)]
    fn as_mut_words(&mut self) -> &mut [Self::Word];
}

macro_rules! implWords {
    ($( ($Word:ty, $SIZE:expr) ),*) => ($(
        impl Sealed for [$Word; $SIZE] {
        }

        impl Words for [$Word; $SIZE] {
            type Word = $Word;

            const LEN: usize = $SIZE;

            #[inline]
            fn splat(word: Self::Word) -> Self {
                [word; Self::LEN]
            }
            #[inline]
            fn boxed(this: Self) -> Box<[Self::Word]> {
                Box::new(this)
            }

            #[inline]
            fn as_ref_words(&self) -> &[Self::Word] {
                &self[..]
            }
            #[inline]
            fn as_mut_words(&mut self) -> &mut [Self::Word] {
                &mut self[..]
            }
        }

        impl FixedBits for [$Word; $SIZE] {
            const SIZE: usize = Self::BITS;
            #[inline]
            fn none() -> Self { [0; $SIZE] }
        }

        impl Bits for [$Word; $SIZE] {
            #[inline]
            fn size(&self) -> usize {
                Self::BITS
            }

            #[inline]
            fn bit(&self, i: usize) -> bool {
                <[$Word] as Bits>::bit(self, i)
            }

            #[inline]
            fn getn<W: Word>(&self, i: usize, n: usize) -> W {
                <[$Word] as Bits>::getn(self, i, n)
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
            fn rank1<R: std::ops::RangeBounds<usize>>(&self, range: R) -> usize {
                self.as_ref().rank1(range)
            }
            #[inline]
            fn rank0<R: std::ops::RangeBounds<usize>>(&self, range: R) -> usize {
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
        }

        impl BitsMut for [$Word; $SIZE] {
            #[inline]
            fn put1(&mut self, i: usize)  {
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
    )*)
}

macro_rules! WordsImpls {
    ($( $BITS:expr ),*) => ($(
        implWords!( (   u8, $BITS /   u8::BITS)
                  , (  u16, $BITS /  u16::BITS)
                  , (  u32, $BITS /  u32::BITS)
                  , (  u64, $BITS /  u64::BITS)
                  , ( u128, $BITS / u128::BITS)
                  );
    )*)
}

const SHORT: usize = 65536;
WordsImpls!((SHORT / 16), (SHORT / 8), (SHORT / 4), (SHORT / 2), SHORT);

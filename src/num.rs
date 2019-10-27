#![allow(clippy::cast_lossless)]

//! Module `num` defines helper traits and functions.

use std::{convert::TryFrom, fmt, hash::Hash, iter::Sum, ops};

use crate::{bits::to_exclusive, num, ops::*};

/// A trait for integral types.
pub trait Int:
    'static
    + Copy
    + Default
    + Clone
    + Eq
    + Ord
    + Hash
    + Sum
    + fmt::Debug
    + fmt::Display
    + fmt::Binary
    + fmt::Octal
    + fmt::UpperHex
    + fmt::LowerHex
    + ops::Add<Output = Self>
    + ops::AddAssign
    + ops::Sub<Output = Self>
    + ops::SubAssign
    + ops::Mul<Output = Self>
    + ops::MulAssign
    + ops::Div<Output = Self>
    + ops::DivAssign
    + ops::Rem<Output = Self>
    + ops::RemAssign
    + ops::BitAnd<Output = Self>
    + ops::BitAndAssign
    + ops::BitOr<Output = Self>
    + ops::BitOrAssign
    + ops::BitXor<Output = Self>
    + ops::BitXorAssign
    + ops::Shl<usize, Output = Self>
    + ops::ShlAssign<usize>
    + ops::Shr<usize, Output = Self>
    + ops::ShrAssign<usize>
    + ops::Not<Output = Self>
    + Sealed
{
    // 0
    #[doc(hidden)]
    const _0: Self;

    // 1
    #[doc(hidden)]
    const _1: Self;

    // Size in bits.
    const BITS: usize;

    // No bits are enabled.
    // count0 == Self::BITS
    const NONE: Self;

    // All bits are enabled.
    // count1 == Self::BITS
    const FULL: Self;
}

/// Unsigned int
pub trait Word:
    Int
    + FixedBits
    + TryFrom<u8>
    + TryFrom<u16>
    + TryFrom<u32>
    + TryFrom<u64>
    + TryFrom<u128>
    + TryFrom<usize>
{
}

macro_rules! implInt {
    ($( ( $Word:ty, $Sint:ty) ),*) => ($(
        impl Int for $Word {
            const _0: Self = 0;
            const _1: Self = 1;

            const NONE: Self = 0;
            const FULL: Self = !0;
            const BITS: usize = std::mem::size_of::<$Word>() * 8;
        }

        impl Int for $Sint {
            const _0: Self = 0;
            const _1: Self = 1;

            const NONE: Self = 0;
            const FULL: Self = !0; // -1
            const BITS: usize = std::mem::size_of::<$Sint>() * 8;
        }

        impl Word for $Word {}

        // impl Sint for $Sint {}

        impl FixedBits for $Word {
            const SIZE: usize = Self::BITS;
            #[inline(always)]
            fn none() -> Self { Self::NONE }
        }

        // impl FixedBits for $Sint {
        //     const SIZE: usize = Self::BITS;
        //     #[inline(always)]
        //     fn none() -> Self { Self::NONE }
        // }

        impl Bits for $Word {
            #[inline(always)]
            fn size(&self) -> usize { Self::BITS }

            #[inline]
            fn bit(&self, i: usize) -> bool { (*self & (1 << i)) != 0 }

            #[inline]
            fn getn<W: Word>(&self, i: usize, len: usize) -> W {
                num::cast((*self >> i) & mask1::<$Word>(len))
            }

            #[inline(always)]
            fn count1(&self) -> usize { self.count_ones()  as usize }
            #[inline(always)]
            fn count0(&self) -> usize { self.count_zeros() as usize }

            #[inline(always)]
            fn all(&self) -> bool { *self == Self::FULL }
            #[inline(always)]
            fn any(&self) -> bool { *self != Self::NONE }

            #[inline(always)]
            fn rank1<R: ops::RangeBounds<usize>>(&self, range: R) -> usize {
                let (i, j) = to_exclusive(&range, Self::BITS).expect("out of bounds");
                (*self & mask::<$Word>(i, j)).count1()
            }
            #[inline(always)]
            fn rank0<R: ops::RangeBounds<usize>>(&self, range: R) -> usize {
                (!*self).rank1(range)
            }

            #[inline(always)]
            fn select1(&self, n: usize) -> Option<usize> { Broadword::broadword(self, n) }
            #[inline(always)]
            fn select0(&self, n: usize) -> Option<usize> { (!*self).select1(n) }
        }

        impl BitsMut for $Word {
            #[inline]
            fn put1(&mut self, i: usize) {
                *self |= 1 << i;
            }
            #[inline]
            fn put0(&mut self, i: usize) {
                *self &= !(1 << i);
            }
            #[inline]
            fn flip(&mut self, i: usize) {
                *self ^= 1 << i;
            }

            // #[inline]
            // fn putn<W: Word>(&mut self, i: usize, len: usize, num: W) {
            //     assert!(len <= W::BITS && i < Self::BITS && i + len <= Self::BITS);
            //     let smask = mask1::<$Word>(len) << i;
            //     *self &= !smask;
            //     *self |= num::try_cast::<W, Self>(num & mask1(len)) << i;
            // }
        }
    )*)
}
#[rustfmt::skip]
implInt!((u8, i8), (u16, i16), (u32, i32), (u64, i64), (u128, i128), (usize, isize));

/// Casts `M` to `N`, panics if cast failed.
#[inline(always)]
pub(crate) fn cast<M: Int, N: Int + TryFrom<M>>(m: M) -> N {
    N::try_from(m).ok().unwrap()
}

/// Binary search to find and return the smallest index k in `[i, j)` at which f(k) is true,
/// assuming that on the range `[i, j)`, f(k) == true implies f(k+1) == true.
///
/// `search` returns the first true index. If there is no such index, returns `j`.
/// `search` calls f(i) only for i in the range `[i, j)`.
pub(crate) fn binary_search<T, F>(start: T, end: T, f: F) -> T
where
    T: Int,
    F: Fn(T) -> bool,
{
    BOUNDS_CHECK!(start < end);

    let mut i = start;
    let mut j = end;

    while i < j {
        let h = i + (j - i) / (T::_1 + T::_1);
        if f(h) {
            j = h; // f(j) == true
        } else {
            i = h + T::_1; // f(i-1) == false
        }
    }
    i // f(i-1) == false && f(i) (= f(j)) == true
}

/// Mask [i, j)
#[inline]
pub(crate) fn mask<T: Int>(i: usize, j: usize) -> T {
    mask1::<T>((j - i) << i)
}

/// Mask [0, i)
#[inline]
pub(crate) fn mask1<T: Int>(i: usize) -> T {
    assert!(i <= T::BITS);
    if i == T::BITS {
        T::FULL
    } else {
        (T::_1 << i) - T::_1
    }
}

// Helper trait to implement `select1` and `select0`
trait Broadword {
    fn broadword(&self, c: usize) -> Option<usize>;
}

impl Broadword for u64 {
    fn broadword(&self, c: usize) -> Option<usize> {
        const X01: u64 = 0x0101_0101_0101_0101;
        const X02: u64 = 0x2020_2020_2020_2020;
        const X33: u64 = 0x3333_3333_3333_3333;
        const X22: u64 = 0x2222_2222_2222_2222;
        const X80: u64 = 0x2010_0804_0201_0080;
        const X81: u64 = 0x2010_0804_0201_0081;
        const X0F: u64 = 0x0f0f_0f0f_0f0f_0f0f;
        const X55: u64 = X22 + X33 + X22 + X33;
        const X8X: u64 = X81 + X80 + X80 + X80;

        #[inline]
        const fn le8(x: u64, y: u64) -> u64 {
            let x8 = X02 + X02 + X02 + X02;
            let xs = (y | x8) - (x & !x8);
            (xs ^ x ^ y) & x8
        }

        #[inline]
        const fn lt8(x: u64, y: u64) -> u64 {
            let x8 = X02 + X02 + X02 + X02;
            let xs = (x | x8) - (y & !x8);
            (xs ^ x ^ !y) & x8
        }

        if c < self.count1() {
            let x = *self;
            let c = c as u64;
            let s0 = x - ((x & X55) >> 1);
            let s1 = (s0 & X33) + ((s0 >> 2) & X33);
            let s2 = ((s1 + (s1 >> 4)) & X0F).wrapping_mul(X01);
            let p0 = (le8(s2, c * X01) >> 7).wrapping_mul(X01);
            let p1 = (p0 >> 53) & !0x7;
            let p2 = p1 as u32;
            let p3 = (s2 << 8).wrapping_shr(p2);
            let p4 = c - (p3 & 0xFF);
            let p5 = lt8(0x0, ((x.wrapping_shr(p2) & 0xFF) * X01) & X8X);
            let s3 = (p5 >> 0x7).wrapping_mul(X01);
            let p6 = (le8(s3, p4 * X01) >> 7).wrapping_mul(X01) >> 56;
            let p7 = p1 + p6;
            // assert!((p7 as usize) < Self::BITS);
            Some(p7 as usize)
        } else {
            None
        }
    }
}

macro_rules! implBroadword {
    ( $( $Ty:ty ),* ) => ($(
        impl Broadword for $Ty {
            #[inline]
            fn broadword(&self, c: usize) -> Option<usize> {
                if c < self.count1() {
                    (*self as u64).select1(c)
                } else {
                    None
                }
            }
        }
    )*)
}
implBroadword!(u8, u16, u32);

impl Broadword for u128 {
    /// ```
    /// use compacts::ops::Bits;
    /// let hi: u64 = 0b_10101010;
    /// let lo: u64 = 0b_01010101;
    /// let n: u128 = ((hi as u128) << 64) | lo as u128;
    /// assert_eq!(n.select1(0), Some(0));
    /// assert_eq!(n.select1(4), Some(65));
    /// ```
    fn broadword(&self, c: usize) -> Option<usize> {
        let slice = [*self as u64, (*self >> 64) as u64];
        slice.as_ref().select1(c)

        // let (hi, lo) = ((*self >> 64) as u64, *self as u64);
        // if c < lo.count1() {
        //     lo.select1(c)
        // } else {
        //     hi.select1(c - lo.count1()).map(|x| x + 64)
        // }
    }
}

impl Broadword for usize {
    #[inline]
    fn broadword(&self, c: usize) -> Option<usize> {
        if c >= self.count1() {
            None
        } else {
            #[cfg(target_pointer_width = "32")]
            {
                (*self as u32).select1(c)
            }
            #[cfg(target_pointer_width = "64")]
            {
                (*self as u64).select1(c)
            }
        }
    }
}

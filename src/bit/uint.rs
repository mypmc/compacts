use crate::{
    bit::ops::*,
    bit::{cast, from_any_bounds, OUT_OF_BOUNDS},
    private,
};

use std::ops::Range;

/// Trait for an unsigned int. This trait is public but sealed.
pub trait Uint:
    'static
    + Copy
    + Eq
    + Ord
    + Default
    + std::ops::Add<Output = Self>
    + std::ops::AddAssign
    + std::ops::Sub<Output = Self>
    + std::ops::SubAssign
    + std::ops::Mul<Output = Self>
    + std::ops::MulAssign
    + std::ops::Div<Output = Self>
    + std::ops::DivAssign
    + std::ops::Rem<Output = Self>
    + std::ops::RemAssign
    + std::ops::Shl<Output = Self>
    + std::ops::ShlAssign
    + std::ops::Shr<Output = Self>
    + std::ops::ShrAssign
    + std::ops::BitAnd<Output = Self>
    + std::ops::BitAndAssign
    + std::ops::BitOr<Output = Self>
    + std::ops::BitOrAssign
    + std::ops::BitXor<Output = Self>
    + std::ops::BitXorAssign
    + std::ops::Not<Output = Self>
    + std::iter::Sum
    + Finite
    + Count
    + Rank
    + Select1
    + Select0
    + Access
    + Assign<u64>
    + Assign<Range<u64>, Output = u64>
    + Read<u8>
    + Read<u16>
    + Read<u32>
    + Read<u64>
    + Read<u128>
    + Read<usize>
    + TryCast<u8>
    + TryCast<u16>
    + TryCast<u32>
    + TryCast<u64>
    + TryCast<u128>
    + TryCast<usize>
    + TryCastFrom<u8>
    + TryCastFrom<u16>
    + TryCastFrom<u32>
    + TryCastFrom<u64>
    + TryCastFrom<u128>
    + TryCastFrom<usize>
    + private::Sealed
{
    const ZERO: Self;

    const ONE: Self;

    fn mask<T: Uint + TryCast<Self>>(i: T) -> Self {
        Self::ONE.shiftl(i) - Self::ONE
    }

    /// Equals to `wrapping_shl`.
    #[doc(hidden)]
    fn shiftl<T: Uint + TryCast<Self>>(&self, i: T) -> Self;

    /// Equals to `wrapping_shr`.
    #[doc(hidden)]
    fn shiftr<T: Uint + TryCast<Self>>(&self, i: T) -> Self;
}

/// Lossless cast that never fail.
pub trait Cast<T>: crate::private::Sealed {
    fn cast(self) -> T;
}

/// Lossless cast that may fail.
pub trait TryCast<T>: crate::private::Sealed {
    fn try_cast(self) -> Option<T>;
}

/// Lossless cast that never fail.
pub trait CastFrom<T>: Sized + crate::private::Sealed {
    fn cast_from(from: T) -> Self;
}

/// Lossless cast that may fail.
pub trait TryCastFrom<T>: Sized + crate::private::Sealed {
    fn try_cast_from(from: T) -> Option<Self>;
}

impl<T: Uint> CastFrom<T> for T {
    fn cast_from(from: T) -> T {
        from
    }
}
impl<T: Uint, U: CastFrom<T>> TryCastFrom<T> for U {
    fn try_cast_from(from: T) -> Option<Self> {
        Some(U::cast_from(from))
    }
}

impl<T: Uint, U: CastFrom<T>> Cast<U> for T {
    fn cast(self) -> U {
        U::cast_from(self)
    }
}

impl<T: Uint, U: TryCastFrom<T>> TryCast<U> for T {
    fn try_cast(self) -> Option<U> {
        U::try_cast_from(self)
    }
}

macro_rules! implUint {
    ($($ty:ty),*) => ($(
        impl Uint for $ty {
            const ZERO: Self = 0;
            const ONE:  Self = 1;

            fn shiftl<T>(&self, i: T) -> Self where T: Uint + TryCast<Self> {
                self.wrapping_shl(cast(i))
            }
            fn shiftr<T>(&self, i: T) -> Self where T: Uint + TryCast<Self> {
                self.wrapping_shr(cast(i))
            }
        }
    )*)
}
implUint!(u8, u16, u32, u64, u128, usize);

macro_rules! implCastFrom {
    ( $large:ty; $( $small:ty ),* ) => ($(
        impl CastFrom<$small> for $large {
            #[allow(clippy::cast_lossless)]
            #[inline]
            fn cast_from(from: $small) -> $large {
                from as $large
            }
        }
    )*)
}
implCastFrom!(u128; u8, u16, u32, u64);
implCastFrom!( u64; u8, u16, u32);
implCastFrom!( u32; u8, u16);
implCastFrom!( u16; u8);

#[cfg(target_pointer_width = "32")]
mod cast_from_for_usize {
    use super::*;
    implCastFrom!(usize; u8, u16, u32);
    implCastFrom!(u128; usize);
    implCastFrom!( u64; usize);
    implCastFrom!( u32; usize);
}
#[cfg(target_pointer_width = "64")]
mod cast_from_for_usize {
    use super::*;
    implCastFrom!(usize; u8, u16, u32, u64);
    implCastFrom!(u128; usize);
    implCastFrom!( u64; usize);
}

macro_rules! implTryCastFrom {
    ( $small:ty; $( $large:ty ),* ) => ($(
        impl TryCastFrom<$large> for $small {
            #[allow(clippy::cast_lossless)]
            #[inline]
            fn try_cast_from(from: $large) -> Option<$small> {
                const MIN: $small = 0;
                const MAX: $small = !MIN;
                if from <= MAX as $large {
                    Some(from as $small)
                } else {
                    None
                }
            }
        }
    )*)
}
implTryCastFrom!(u64; u128);
implTryCastFrom!(u32; u128, u64);
implTryCastFrom!(u16; u128, u64, u32);
implTryCastFrom!( u8; u128, u64, u32, u16);

#[cfg(target_pointer_width = "32")]
mod try_cast_from_for_usize {
    use super::*;
    implTryCastFrom!(usize; u128);
    implTryCastFrom!(u16; usize);
    implTryCastFrom!( u8; usize);
}
#[cfg(target_pointer_width = "64")]
mod try_cast_from_for_usize {
    use super::*;
    implTryCastFrom!(usize; u128);
    implTryCastFrom!(u32; usize);
    implTryCastFrom!(u16; usize);
    implTryCastFrom!( u8; usize);
}

macro_rules! impls {
    ($($ty:ty),*) => ($(
        impl Finite for $ty {
            #[allow(clippy::cast_lossless)]
            const BITS: u64 = std::mem::size_of::<$ty>() as u64 * 8;
            #[inline]
            fn empty() -> Self { 0 }
        }

        impl Count for $ty {
            #[inline]
            fn bits(&self) -> u64 {
                Self::BITS
            }
            #[inline]
            fn count1(&self) -> u64 {
                u64::from(self.count_ones())
            }
            #[inline]
            fn count0(&self) -> u64 {
                u64::from(self.count_zeros())
            }
        }

        impl Rank for $ty {
            fn rank1(&self, i: u64) -> u64 {
                assert!(i <= Self::BITS, OUT_OF_BOUNDS);
                if i == Self::BITS {
                    self.count1()
                } else {
                    let mask = *self & Self::mask(i);
                    mask.count1()
                }
            }

            fn rank0(&self, i: u64) -> u64 {
                assert!(i <= Self::BITS, OUT_OF_BOUNDS);
                (!self).rank1(i)
            }
        }

        impl Access for $ty {
            #[inline]
            fn access(&self, i: u64) -> bool {
                (*self & Self::ONE.shiftl(i)) != Self::ZERO
            }
        }

        impl Assign<u64> for $ty {
            type Output = ();
            #[inline]
            fn set1(&mut self, i: u64) -> Self::Output {
                assert!(i < Self::BITS, OUT_OF_BOUNDS);
                *self |= Self::ONE.shiftl(i);
            }

            #[inline]
            fn set0(&mut self, i: u64) -> Self::Output {
                assert!(i < Self::BITS, OUT_OF_BOUNDS);
                *self &= !Self::ONE.shiftl(i);
            }

            // #[inline]
            // fn flip(&mut self, i: u64) -> Self::Output {
            //     assert!(i < Self::BITS, OUT_OF_BOUNDS);
            //     *self ^= Self::bit(cast(i));
            // }
        }

        impl Assign<Range<u64>> for $ty {
            type Output = u64;

            fn set1(&mut self, r: Range<u64>) -> Self::Output {
                let i = r.start;
                let j = r.end;
                if i >= j {
                    0
                } else {
                    assert!(i < Self::BITS && j <= Self::BITS);
                    let head = (!<$ty as Uint>::ZERO) << i;
                    let last = (!<$ty as Uint>::ZERO).shiftr(Self::BITS - j);
                    let ones = self.count1();
                    *self |= head & last;
                    self.count1() - ones
                }
            }

            fn set0(&mut self, r: Range<u64>) -> Self::Output {
                let i = r.start;
                let j = r.end;
                if i >= j {
                    0
                } else {
                    assert!(i < Self::BITS && j <= Self::BITS);
                    let head = (!<$ty as Uint>::ZERO) << i;
                    let last = (!<$ty as Uint>::ZERO).shiftr(Self::BITS - j);
                    let ones = self.count1();
                    *self &= !(head & last);
                    ones - self.count1()
                }
            }
        }

        impl<W: Uint> Read<W> for $ty {
            fn read<R: std::ops::RangeBounds<u64>>(&self, r: R) -> W {
                let r = from_any_bounds(&r, self.bits());
                if r.start == 0 && r.end == Self::BITS {
                    cast(*self)
                } else {
                    let i = r.start;
                    let j = r.end;
                    assert!(i < j && j - i <= W::BITS && i < self.bits() && j <= self.bits());
                    // let self = 01101010, i = 1 and j = 6
                    // head: 11111110
                    let head = (!<$ty as Uint>::ZERO).shiftl(i);
                    // last: 00111111
                    let last = (!<$ty as Uint>::ZERO).shiftr(Self::BITS - j);
                    // mask: 00111110
                    let mask = head & last;
                    cast::<$ty, W>(*self & mask).shiftr(i)
                }
            }
        }
    )*)
}
impls!(u8, u16, u32, u64, u128, usize);

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
fn le8(x: u64, y: u64) -> u64 {
    let x8 = X02 + X02 + X02 + X02;
    let xs = (y | x8) - (x & !x8);
    (xs ^ x ^ y) & x8
}

#[inline]
fn lt8(x: u64, y: u64) -> u64 {
    let x8 = X02 + X02 + X02 + X02;
    let xs = (x | x8) - (y & !x8);
    (xs ^ x ^ !y) & x8
}

impl Select1 for u64 {
    /// # Examples
    ///
    /// ```
    /// use compacts::bit::ops::Select1;
    /// let n = 0b_00000100_10010000_u64;
    /// assert_eq!(n.select1(0), Some(4));
    /// assert_eq!(n.select1(1), Some(7));
    /// assert_eq!(n.select1(2), Some(10));
    /// assert_eq!(n.select1(3), None);
    ///
    /// use compacts::bit::ops::Rank;
    /// assert_eq!(n.rank1(n.select1(0).unwrap()), 0);
    /// assert_eq!(n.rank1(n.select1(1).unwrap()), 1);
    /// assert_eq!(n.rank1(n.select1(2).unwrap()), 2);
    /// ```
    #[allow(clippy::cast_lossless)]
    fn select1(&self, c: u64) -> Option<u64> {
        if c < self.count1() {
            let x = self;
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
            assert!(p7 < Self::BITS);
            Some(p7)
        } else {
            None
        }
    }
}

impl Select1 for u128 {
    /// # Examples
    ///
    /// ```
    /// use compacts::bit::ops::Select1;
    /// let n: u128 = (0b_00001100_u128 << 64) | 0b_00000100_u128;
    /// assert_eq!(n.select1(0), Some(2));
    /// assert_eq!(n.select1(1), Some(66));
    /// assert_eq!(n.select1(2), Some(67));
    /// assert_eq!(n.select1(3), None);
    /// ```
    fn select1(&self, mut c: u64) -> Option<u64> {
        let hi = (*self >> 64) as u64;
        let lo = *self as u64;
        if c < lo.count1() {
            return lo.select1(c);
        }
        c -= lo.count1();
        hi.select1(c).map(|x| x + 64)
    }
}

macro_rules! implSelect1 {
    ( $( $ty:ty ),* ) => ($(
        impl Select1 for $ty {
            #[allow(clippy::cast_lossless)]
            #[inline]
            fn select1(&self, c: u64) -> Option<u64> {
                if c < self.count1() {
                    (*self as u64).select1(c)
                } else {
                    None
                }
            }
        }
    )*)
}
macro_rules! implSelect0 {
    ($($ty:ty),*) => ($(
        impl Select0 for $ty {
            #[inline]
            fn select0(&self, c: u64) -> Option<u64> {
                if c < self.count0() {
                    (!self).select1(c)
                } else {
                    None
                }
            }
        }
    )*)
}
implSelect1!(u8, u16, u32, usize);
implSelect0!(u8, u16, u32, u64, u128, usize);

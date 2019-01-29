use crate::{bits::*, private};

use std::ops::Range;

/// Trait for an unsigned int. This trait is public but sealed.
pub trait UnsignedInt:
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
    + TryCastInto<u8>
    + TryCastInto<u16>
    + TryCastInto<u32>
    + TryCastInto<u64>
    + TryCastInto<u128>
    + TryCastInto<usize>
    + TryCastFrom<u8>
    + TryCastFrom<u16>
    + TryCastFrom<u32>
    + TryCastFrom<u64>
    + TryCastFrom<u128>
    + TryCastFrom<usize>
    + FiniteBits
    + Count
    + Rank
    + Select1
    + Select0
    + Access
    + Assign<u64>
    + Assign<Range<u64>>
    + private::Sealed
{
    const ZERO: Self;

    fn bit(i: Self) -> Self;

    fn mask(i: Self) -> Self;
}

/// Lossless cast that never fail.
pub trait CastInto<T>: crate::private::Sealed {
    fn cast_into(self) -> T;
}

/// Lossless cast that may fail.
pub trait TryCastInto<T>: crate::private::Sealed {
    fn try_cast_into(self) -> Option<T>;
}

/// Lossless cast that never fail.
pub trait CastFrom<T>: Sized + crate::private::Sealed {
    fn cast_from(from: T) -> Self;
}

/// Lossless cast that may fail.
pub trait TryCastFrom<T>: Sized + crate::private::Sealed {
    fn try_cast_from(from: T) -> Option<Self>;
}

impl<T: UnsignedInt> CastFrom<T> for T {
    fn cast_from(from: T) -> T {
        from
    }
}
impl<T: UnsignedInt, U: CastFrom<T>> TryCastFrom<T> for U {
    fn try_cast_from(from: T) -> Option<Self> {
        Some(U::cast_from(from))
    }
}

impl<T: UnsignedInt, U: CastFrom<T>> CastInto<U> for T {
    fn cast_into(self) -> U {
        U::cast_from(self)
    }
}

impl<T: UnsignedInt, U: TryCastFrom<T>> TryCastInto<U> for T {
    fn try_cast_into(self) -> Option<U> {
        U::try_cast_from(self)
    }
}

macro_rules! implUnsignedInt {
    ($($ty:ty),*) => ($(
        impl UnsignedInt for $ty {
            const ZERO: Self = 0;
            fn bit(i: Self) -> Self { 1 << i }
            fn mask(i: Self) -> Self { (1 << i) - 1 }
        }
    )*)
}
implUnsignedInt!(u8, u16, u32, u64, u128, usize);

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
        impl FiniteBits for $ty {
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
                    let mask = *self & Self::mask(ucast(i));
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
                (*self & Self::bit(ucast(i))) != Self::ZERO
            }
        }

        impl Assign<u64> for $ty {
            type Output = ();
            #[inline]
            fn set1(&mut self, i: u64) -> Self::Output {
                assert!(i < Self::BITS, OUT_OF_BOUNDS);
                *self |= Self::bit(ucast(i));
            }

            #[inline]
            fn set0(&mut self, i: u64) -> Self::Output {
                assert!(i < Self::BITS, OUT_OF_BOUNDS);
                *self &= !Self::bit(ucast(i));
            }

            // #[inline]
            // fn flip(&mut self, i: u64) -> Self::Output {
            //     assert!(i < Self::BITS, OUT_OF_BOUNDS);
            //     *self ^= Self::bit(ucast(i));
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
                    let head = (!<$ty as UnsignedInt>::ZERO) << (i % Self::BITS);
                    let last = (!<$ty as UnsignedInt>::ZERO).wrapping_shr(ucast(Self::BITS - j % Self::BITS));
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
                    let head = (!<$ty as UnsignedInt>::ZERO) << (i % Self::BITS);
                    let last = (!<$ty as UnsignedInt>::ZERO).wrapping_shr(ucast(Self::BITS - j % Self::BITS));
                    let ones = self.count1();
                    *self &= !(head & last);
                    ones - self.count1()
                }
            }
        }

        // impl MaskAssign<&'_ $ty> for $ty {
        //     fn intersection(&mut self, that: &$ty) {
        //         *self &= *that;
        //     }
        //     fn union(&mut self, that: &$ty) {
        //         *self |= *that;
        //     }
        //     fn difference(&mut self, that: &$ty) {
        //         *self &= !*that;
        //     }
        //     fn symmetric_difference(&mut self, that: &$ty) {
        //         *self ^= *that;
        //     }
        // }
        // impl MaskAssign<Cow<'_, $ty>> for $ty {
        //     fn intersection(&mut self, that: Cow<'_, $ty>) {
        //         *self &= *that;
        //     }
        //     fn union(&mut self, that: Cow<'_, $ty>) {
        //         *self |= *that;
        //     }
        //     fn difference(&mut self, that: Cow<'_, $ty>) {
        //         *self &= !*that;
        //     }
        //     fn symmetric_difference(&mut self, that: Cow<'_, $ty>) {
        //         *self ^= *that;
        //     }
        // }
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
    /// use compacts::bits::ops::Select1;
    /// let n = 0b_00000100_10010000_u64;
    /// assert_eq!(n.select1(0), Some(4));
    /// assert_eq!(n.select1(1), Some(7));
    /// assert_eq!(n.select1(2), Some(10));
    /// assert_eq!(n.select1(3), None);
    ///
    /// use compacts::bits::ops::Rank;
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
    /// use compacts::bits::ops::Select1;
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

///// `Block<U>` is a boxed slice of `U`.
///// It is almost same with `[U]` where U is an UnsignedInt, except that `Block<U>` implements `FiniteBits`.
/////
///// Currently bit size of `Block<U>` is fixed, but if const generics is stabilized,
///// we will change type signature to `Block<U, LEN>`.
//#[derive(Clone, Debug, PartialEq, Eq)]
//pub struct Block<U> {
//    ones: u32,
//    data: Option<Box<[U]>>,
//}

//impl<U: UnsignedInt> Default for Block<U> {
//    fn default() -> Self {
//        Block {
//            ones: 0,
//            data: None,
//        }
//    }
//}

//impl<U: UnsignedInt> Block<U> {
//    const LEN: usize = (Self::BITS / U::BITS) as usize;

//    /// Constructs a new instance with each element initialized to value.
//    pub fn splat(value: U) -> Self {
//        let ones = ucast::<u64, u32>(value.count1()) * ucast::<usize, u32>(Self::LEN);
//        let data = Some(vec![value; Self::LEN].into_boxed_slice());
//        Block { ones, data }
//    }
//}

//impl<U: UnsignedInt> From<&'_ [U]> for Block<U> {
//    fn from(slice: &'_ [U]) -> Self {
//        Self::from(slice.to_vec())
//    }
//}
//impl<U: UnsignedInt> From<Vec<U>> for Block<U> {
//    fn from(mut vec: Vec<U>) -> Self {
//        if vec.is_empty() {
//            Block::empty()
//        } else {
//            vec.resize(Self::LEN, U::ZERO);
//            let ones = ucast::<u64, u32>(vec.count1());
//            let data = Some(vec.into_boxed_slice());
//            Block { ones, data }
//        }
//    }
//}

//impl<U: UnsignedInt> FiniteBits for Block<U> {
//    const BITS: u64 = 1 << 16;
//    fn empty() -> Self {
//        Self::default()
//    }
//}

//impl<U: UnsignedInt> Count for Block<U> {
//    fn bits(&self) -> u64 {
//        Self::BITS
//    }
//    fn count1(&self) -> u64 {
//        u64::from(self.ones)
//    }
//}

//impl<U: UnsignedInt> Access for Block<U> {
//    /// Uest bit at `i`.
//    fn access(&self, i: u64) -> bool {
//        assert!(i < self.bits(), OUT_OF_BOUNDS);
//        self.data.as_ref().map_or(false, |vec| vec.access(i))
//    }
//    fn iterate<'a>(&'a self) -> Box<dyn Iterator<Item = u64> + 'a> {
//        if let Some(slice) = self.data.as_ref() {
//            slice.iterate()
//        } else {
//            Box::new(std::iter::empty())
//        }
//    }
//}

//impl<U: UnsignedInt> Rank for Block<U> {
//    fn rank1(&self, i: u64) -> u64 {
//        assert!(i <= self.bits(), OUT_OF_BOUNDS);
//        self.data.as_ref().map_or(0, |vec| vec.rank1(i))
//    }
//}

//impl<U: UnsignedInt> Select1 for Block<U> {
//    fn select1(&self, n: u64) -> Option<u64> {
//        self.data.as_ref().and_then(|vec| vec.select1(n))
//    }
//}
//impl<U: UnsignedInt> Select0 for Block<U> {
//    fn select0(&self, n: u64) -> Option<u64> {
//        self.data.as_ref().map_or(Some(n), |vec| vec.select0(n))
//    }
//}

//impl<U: UnsignedInt> Assign<u64> for Block<U> {
//    type Output = ();

//    fn set1(&mut self, i: u64) -> Self::Output {
//        assert!(i < self.bits(), OUT_OF_BOUNDS);

//        let (i, o) = divmod::<usize>(i, U::BITS);
//        if let Some(vec) = self.data.as_mut() {
//            if !vec[i].access(o) {
//                vec[i].set1(o);
//                self.ones += 1;
//            }
//        } else {
//            self.ones += 1;
//            let mut vec = vec![U::ZERO; Self::LEN];
//            vec[i].set1(o);
//            self.data = Some(vec.into_boxed_slice());
//        }
//    }

//    fn set0(&mut self, i: u64) -> Self::Output {
//        assert!(i < self.bits(), OUT_OF_BOUNDS);
//        let (i, o) = divmod::<usize>(i, U::BITS);
//        if let Some(vec) = self.data.as_mut() {
//            if vec[i].access(o) {
//                self.ones -= 1;
//                vec[i].set0(o);
//            }
//        }
//    }
//}

//impl<U: UnsignedInt> Assign<Range<u64>> for Block<U>
//where
//    [U]: Assign<Range<u64>, Output = u64>,
//{
//    type Output = u64;

//    /// # Examples
//    ///
//    /// ```
//    /// use compacts::bits::{Block, Assign};
//    /// let mut map = Block::from(vec![0b_00000000u8, 0b_00000000]);
//    /// map.set1(0..3);
//    /// assert_eq!(map, Block::from(vec![0b_00000111u8, 0b_00000000]));
//    /// map.set1(14..18);
//    /// assert_eq!(map, Block::from(vec![0b_00000111u8, 0b_11000000, 0b_00000011]));
//    /// ```
//    fn set1(&mut self, r: Range<u64>) -> Self::Output {
//        if r.start >= r.end {
//            return 0;
//        }

//        if let Some(vec) = self.data.as_mut() {
//            let out = vec.set1(r);
//            self.ones += ucast::<u64, u32>(out);
//            out
//        } else {
//            let mut vec = vec![U::ZERO; Self::LEN];
//            let out = vec.set1(r);
//            self.ones = ucast(out);
//            self.data = Some(vec.into_boxed_slice());
//            out
//        }
//    }

//    fn set0(&mut self, r: Range<u64>) -> Self::Output {
//        if r.start >= r.end {
//            return 0;
//        }
//        if let Some(vec) = self.data.as_mut() {
//            let out = vec.set0(r);
//            self.ones -= ucast::<u64, u32>(out);
//            out
//        } else {
//            0
//        }
//    }
//}

//impl<'a, U: UnsignedInt> std::ops::BitAndAssign<Cow<'a, Block<U>>> for Block<U> {
//    fn bitand_assign(&mut self, cow: Cow<'a, Block<U>>) {
//        self.bitand_assign(cow.as_ref());
//    }
//}
//impl<'b, U: UnsignedInt> std::ops::BitAndAssign<&'b Block<U>> for Block<U> {
//    fn bitand_assign(&mut self, that: &'b Block<U>) {
//        match (self.data.as_mut(), that.data.as_ref()) {
//            (Some(lhs), Some(rhs)) => {
//                assert_eq!(lhs.len(), rhs.len());
//                let mut ones = 0;
//                for (x, y) in lhs.iter_mut().zip(rhs.iter()) {
//                    *x &= *y;
//                    ones += x.count1();
//                }
//                self.ones = ucast(ones);
//            }
//            _ => {
//                self.data = None;
//            }
//        }
//    }
//}

//impl<'a, U: UnsignedInt> std::ops::BitOrAssign<Cow<'a, Block<U>>> for Block<U> {
//    fn bitor_assign(&mut self, cow: Cow<'a, Block<U>>) {
//        self.bitor_assign(cow.as_ref());
//    }
//}
//impl<'b, U: UnsignedInt> std::ops::BitOrAssign<&'b Block<U>> for Block<U> {
//    fn bitor_assign(&mut self, that: &'b Block<U>) {
//        match (self.data.as_mut(), that.data.as_ref()) {
//            (None, Some(vec)) => {
//                let mut dst = vec![U::ZERO; vec.len()];
//                dst.copy_from_slice(&vec[..]);
//                self.data = Some(dst.into_boxed_slice());
//            }
//            (Some(lhs), Some(rhs)) => {
//                assert_eq!(lhs.len(), rhs.len());
//                let mut ones = 0;
//                for (x, y) in lhs.iter_mut().zip(rhs.iter()) {
//                    *x |= *y;
//                    ones += x.count1();
//                }
//                self.ones = ucast(ones);
//            }
//            _ => {}
//        }
//    }
//}

//impl<'a, U: UnsignedInt> std::ops::BitXorAssign<Cow<'a, Block<U>>> for Block<U> {
//    fn bitxor_assign(&mut self, cow: Cow<'a, Block<U>>) {
//        self.bitxor_assign(cow.as_ref());
//    }
//}
//impl<'b, U: UnsignedInt> std::ops::BitXorAssign<&'b Block<U>> for Block<U> {
//    fn bitxor_assign(&mut self, that: &'b Block<U>) {
//        match (self.data.as_mut(), that.data.as_ref()) {
//            (None, Some(buf)) => {
//                let mut dst = vec![U::ZERO; buf.len()];
//                dst.copy_from_slice(&buf[..]);
//                self.data = Some(dst.into_boxed_slice());
//            }
//            (Some(lhs), Some(rhs)) => {
//                assert_eq!(lhs.len(), rhs.len());
//                let mut ones = 0;
//                for (x, y) in lhs.iter_mut().zip(rhs.iter()) {
//                    *x ^= *y;
//                    ones += x.count1();
//                }
//                self.ones = ucast(ones);
//            }
//            _ => {}
//        }
//    }
//}

//impl<U: UnsignedInt> std::ops::Not for Block<U> {
//    type Output = Block<U>;
//    fn not(self) -> Self::Output {
//        match self.data {
//            Some(mut vec) => {
//                let ones = {
//                    let mut acc = 0;
//                    for v in vec.iter_mut() {
//                        *v = !*v;
//                        acc += v.count1();
//                    }
//                    acc
//                };
//                Block {
//                    ones: ucast(ones),
//                    data: if ones > 0 { Some(vec) } else { None },
//                }
//            }
//            None => Self::splat(!U::ZERO),
//        }
//    }
//}

//impl<U: UnsignedInt> std::ops::Not for &'_ Block<U> {
//    type Output = Block<U>;
//    fn not(self) -> Self::Output {
//        match self.data {
//            Some(ref vec) => {
//                let mut out = vec![U::ZERO; Block::<U>::LEN];
//                let mut acc = 0;
//                for (a, b) in out.iter_mut().zip(vec.iter()) {
//                    *a = !*b;
//                    acc += a.count1();
//                }
//                Block {
//                    ones: ucast(acc),
//                    data: if acc > 0 {
//                        Some(out.into_boxed_slice())
//                    } else {
//                        None
//                    },
//                }
//            }
//            None => Block::splat(!U::ZERO),
//        }
//    }
//}

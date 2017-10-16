use std::ops;

/// `PopCount` is a trait for `PopCount` or `HammingWeight`.
pub trait PopCount<T>
where
    T: ops::Sub<Output = T> + Copy,
{
    const SIZE: T;

    fn count1(&self) -> T {
        Self::SIZE - self.count0()
    }

    fn count0(&self) -> T {
        Self::SIZE - self.count1()
    }
}

/// `Rank` is a generalization of `PopCount`.
pub trait Rank<T>
where
    T: ops::Sub<Output = T> + Copy,
{
    /// Returns occurences of non-zero bit in `[0, i)` for `0 < i`.
    /// `rank1(i)` should be equal to `i - self.rank0(i)`.
    fn rank1(&self, i: T) -> T {
        i - self.rank0(i)
    }

    /// Returns occurences of zero bit in `[0, i)` for `0 < i`.
    /// `rank0(i)` should be equal to `i - self.rank1(i)`.
    fn rank0(&self, i: T) -> T {
        i - self.rank1(i)
    }
}

macro_rules! impl_PopCount {
    ( $( $out:ty ),* ) => ($(
        impl PopCount<$out> for u64 {
            const SIZE: $out = 64;
            #[cfg_attr(feature = "cargo-clippy", allow(cast_lossless))]
            fn count1(&self) -> $out {
                self.count_ones() as $out
            }
        }
    )*)
}
impl_PopCount!(u64, u32, u16, u8);

impl Rank<u32> for u64 {
    fn rank1(&self, i: u32) -> u32 {
        if i == 0 {
            return 0;
        }
        if i >= <u64 as PopCount<u32>>::SIZE {
            self.count1()
        } else {
            let mask = (1 << i) - 1;
            (self & mask).count1()
        }
    }
}

pub trait Select1<T> {
    /// Returns the position of 'c+1'th appearance of non-zero bit.
    fn select1(&self, c: T) -> Option<T>;
}

pub trait Select0<T> {
    /// Returns the position of 'c+1'th appearance of non-zero bit.
    fn select0(&self, c: T) -> Option<T>;
}

macro_rules! impl_Select {
    ( $( $pos:ty ),* ) => ($(
        #[cfg_attr(feature = "cargo-clippy", allow(cast_lossless))]
        impl Select1<$pos> for u64 {
            #[inline]
            fn select1(&self, c: $pos) -> Option<$pos> {
                if c >= self.count_ones() as $pos {
                    return None;
                }
                let width = 64;
                assert!(c < width as $pos);
                let x = self;
                let w = u64::from(c);
                let s0 = x - ((x & X55) >> 1);
                let s1 = (s0 & X33) + ((s0 >> 2) & X33);
                let s2 = ((s1 + (s1 >> 4)) & X0F).wrapping_mul(X01);
                let p0 = (le8(s2, (w * X01)) >> 7).wrapping_mul(X01);
                let p1 = (p0 >> 53) & !0x7;
                let p2 = p1 as u32;
                let p3 = (s2 << 8).wrapping_shr(p2);
                let p4 = w - (p3 & 0xFF);
                let p5 = lt8(0x0, ((x.wrapping_shr(p2) & 0xFF) * X01) & X8X);
                let s3 = (p5 >> 0x7).wrapping_mul(X01);
                let p6 = (le8(s3, (p4 * X01)) >> 7).wrapping_mul(X01) >> 56;
                let ix = p1 + p6;
                if ix >= width as u64 { None } else { Some(ix as $pos) }
            }
        }

        impl Select0<$pos> for u64 {
            #[inline]
            fn select0(&self, c: $pos) -> Option<$pos> {
                (!self).select1(c)
            }
        }
    )*)
}
impl_Select!(u64, u32, u16, u8);

const X01: u64 = 0x0101_0101_0101_0101;
const X02: u64 = 0x2020_2020_2020_2020;
const X33: u64 = 0x3333_3333_3333_3333;
const X22: u64 = 0x2222_2222_2222_2222;
const X80: u64 = 0x2010_0804_0201_0080;
const X81: u64 = 0x2010_0804_0201_0081;
const X0F: u64 = 0x0f0f_0f0f_0f0f_0f0f;
const X55: u64 = X22 + X33 + X22 + X33;
const X8X: u64 = X81 + X80 + X80 + X80;

fn le8(x: u64, y: u64) -> u64 {
    let x8 = X02 + X02 + X02 + X02;
    let xs = (y | x8) - (x & !x8);
    (xs ^ x ^ y) & x8
}

fn lt8(x: u64, y: u64) -> u64 {
    let x8 = X02 + X02 + X02 + X02;
    let xs = (x | x8) - (y & !x8);
    (xs ^ x ^ !y) & x8
}

pub trait Dict<T>: ops::Index<T>
where
    <Self as ops::Index<T>>::Output: PartialEq<Self::Item>,
{
    /// Associated items to this dictionary.
    type Item;

    /// Result type of `select`.
    type Index;

    /// Result type of `rank`.
    type Count;

    /// Returns count of `Item` in `0..=i`.
    fn rank(&self, item: &Self::Item, i: Self::Index) -> Self::Count;

    /// Returns the position of the `c+1`-th appearance of `Item`.
    fn select(&self, item: &Self::Item, c: Self::Count) -> Option<Self::Index>;
}

// pub trait BitDict<T: UnsignedInt>
//     : ops::Index<T, Output = bool> + Rank<T> + Select0<T, Index = T> + Select1<T, Index = T>
//     {
//     fn rank0(&self, T) -> Self::Count;
//     fn rank1(&self, T) -> Self::Count;
//     fn select0(&self, c: T) -> Option<<Self as Select0<T>>::Index>;
//     fn select1(&self, c: T) -> Option<<Self as Select1<T>>::Index>;
// }

// impl<T, U> BitDict<T> for U
// where
//     T: UnsignedInt,
//     U: ops::Index<T, Output = bool> + Rank<T> + Select0<T, Index = T> + Select1<T, Index = T>,
// {
//     fn rank0(&self, i: T) -> Self::Count {
//         <Self as Rank<T>>::rank0(self, i)
//     }
//     fn rank1(&self, i: T) -> Self::Count {
//         <Self as Rank<T>>::rank1(self, i)
//     }

//     fn select0(&self, c: T) -> Option<<Self as Select0<T>>::Index> {
//         <Self as Select0<T>>::select0(self, c)
//     }
//     fn select1(&self, c: T) -> Option<<Self as Select1<T>>::Index> {
//         <Self as Select1<T>>::select1(self, c)
//     }
// }

// impl<T, U> Dict<T> for U
// where
//     T: UnsignedInt,
//     U: BitDict<T>,
// {
//     type Item = bool;

//     type Index = U::Index;
//     type Count = U::Count;

//     // fn size(&self) -> Self::Rank {<Self as Rank<T>>::SIZE}

//     fn rank(&self, item: &Self::Item, i: Self::Index) -> Self::Count {
//         if *item {
//             BitDict::rank1(self, i)
//         } else {
//             BitDict::rank0(self, i)
//         }
//     }

//     fn select(&self, item: &Self::Item, c: Self::Count) -> Option<Self::Index> {
//         if *item {
//             BitDict::select1(self, c)
//         } else {
//             BitDict::select0(self, c)
//         }
//     }
// }

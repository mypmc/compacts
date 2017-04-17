// Constant sized bits.
pub trait Bits {
    /// Size of this representation.
    const SIZE: u64;

    /// The value with all bits unset.
    fn none() -> Self;

    /// Count non-zero bits.
    // REQUIRES: ones() <= Self::SIZE
    fn ones(&self) -> u64 {
        Self::SIZE - self.zeros()
    }

    /// Count zero bits.
    // REQUIRES: zeros() <= Self::SIZE
    fn zeros(&self) -> u64 {
        Self::SIZE - self.ones()
    }
}
pub trait Rank0<T = usize>: Bits {
    /// Count how many zero bits there are up to a given position
    fn rank0(&self, i: T) -> u64;
}
pub trait Rank1<T = usize>: Bits {
    /// Count how many non-zero bits there are up to a given position
    fn rank1(&self, i: T) -> u64;
}

pub trait Select0<T = usize>: Bits {
    /// Return the 'c+1'th zero bit's index.
    fn select0(&self, c: T) -> Option<u64>;
}
pub trait Select1<T = usize>: Bits {
    /// Return the 'c+1'th non-zero bit's index.
    fn select1(&self, c: T) -> Option<u64>;
}

macro_rules! impl_bits {
    ( $( ($type: ty, $size: expr) ),* ) => ($(
        impl Bits for $type {
            const SIZE: u64 = $size;
            #[inline] fn none() -> Self { 0 }
            #[inline] fn ones(&self) -> u64 {
                let ones = self.count_ones();
                debug_assert!(ones as u64 <= Self::SIZE);
                ones as u64
            }
        }
    )*)
}
impl_bits!((u64, 64), (u32, 32), (u16, 16), (u8, 8));
#[cfg(target_pointer_width = "32")]
impl_bits!{(usize, 32)}
#[cfg(target_pointer_width = "64")]
impl_bits!{(usize, 64)}

impl Bits for bool {
    const SIZE: u64 = 1;
    fn none() -> Self {
        false
    }
    fn ones(&self) -> u64 {
        if *self { 1 } else { 0 }
    }
}

macro_rules! impl_rank9 {
    ( $( ($type: ty, $key: ty) ),* ) => ($(
        impl Rank0<$key> for $type {
            #[inline]
            fn rank0(&self, i: $key) -> u64 {
                let rank1 = self.rank1(i);
                i as u64 - rank1
            }
        }
        impl Rank1<$key> for $type {
            #[inline]
            fn rank1(&self, i: $key) -> u64 {
                let rank = if i as u64 >= Self::SIZE {
                    self.ones()
                } else {
                    let this = *self;
                    (this & ((1 << i) - 1)).ones()
                };
                rank
            }
        }
    )*)
}
macro_rules! impl_rank9_all {
    ( $( $type: ty ),* ) => ($(
        impl_rank9!(($type, u64), ($type, u32), ($type, u16), ($type, u8), ($type, usize));
    )*)
}
impl_rank9_all!(u64, u32, u16, u8, usize);

const X01: u64 = 0x0101010101010101;
const X02: u64 = 0x2020202020202020;
const X33: u64 = 0x3333333333333333;
const X22: u64 = 0x2222222222222222;
const X80: u64 = 0x2010080402010080;
const X81: u64 = 0x2010080402010081;
const X0F: u64 = 0x0f0f0f0f0f0f0f0f;
const X55: u64 = X22 + X33 + X22 + X33;
const X8X: u64 = X81 + X80 + X80 + X80;

macro_rules! impl_select9 {
    ( $( ( $type: ty, $key: ty ) ),* ) => ($(
        impl Select1<$key> for $type {
            #[inline]
            fn select1(&self, c: $key) -> Option<u64> {
                let w = c as u64;
                let x = *self as u64;
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
                let p6 = (le8(s3, (p4 as u64 * X01)) >> 7).wrapping_mul(X01) >> 56;
                let p = p1 + p6;
                if p >= Self::SIZE { None } else { Some(p) }
            }
        }
        impl Select0<$key> for $type {
            #[inline]
            fn select0(&self, c: $key) -> Option<u64> { (!*self).select1(c) }
        }
    )*)
}
macro_rules! impl_select9_all {
    ( $( $type: ty ),* ) => ($(
        impl_select9!(($type, u64), ($type, u32), ($type, u16), ($type, u8), ($type, usize));
    )*)
}
impl_select9_all!(u64, u32, u16, u8, usize);

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

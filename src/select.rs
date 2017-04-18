use super::Bits;

pub trait Select0<T = usize> {
    /// Return the 'c+1'th zero bit's index.
    fn select0(&self, c: T) -> Option<u64>;
}
pub trait Select1<T = usize> {
    /// Return the 'c+1'th non-zero bit's index.
    fn select1(&self, c: T) -> Option<u64>;
}

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
impl_select9_all!(u64, u32, u16, usize);

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

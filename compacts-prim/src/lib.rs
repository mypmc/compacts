#![feature(associated_consts)]

extern crate num;

pub use num::{Unsigned, PrimInt as Int, Bounded, One, Zero, NumCast as Cast};

pub trait UnsignedInt: Unsigned + Int {
    const WIDTH: usize;

    #[inline]
    fn min_value() -> Self {
        <Self as Bounded>::min_value()
    }
    #[inline]
    fn max_bound() -> Self {
        <Self as Bounded>::max_value()
    }

    #[inline]
    fn count_ones(self) -> u32 {
        <Self as Int>::count_ones(self)
    }
    #[inline]
    fn count_zeros(self) -> u32 {
        <Self as Int>::count_zeros(self)
    }

    #[inline]
    fn succ(&self) -> Self {
        *self + Self::one()
    }
    #[inline]
    fn pred(&self) -> Self {
        *self - Self::one()
    }

    #[inline]
    fn incr(&mut self) {
        *self = self.succ()
    }
    #[inline]
    fn decr(&mut self) {
        *self = self.pred()
    }
}

macro_rules! impl_UnsignedInt {
    ( $( ( $this:ty, $size:expr ) ),* ) => ($(
        impl UnsignedInt for $this {
            const WIDTH: usize = $size;
        }
    )*)
}

impl_UnsignedInt!((u64, 64), (u32, 32), (u16, 16), (u8, 8));
#[cfg(target_pointer_width = "32")]
impl_UnsignedInt!((usize, 32));
#[cfg(target_pointer_width = "64")]
impl_UnsignedInt!((usize, 64));

/// Find the smallest index i in range at which f(i) is true, assuming that
/// f(i) == true implies f(i+1) == true.
pub fn search<T, F>(range: &::std::ops::Range<T>, f: F) -> T
    where T: UnsignedInt,
          F: Fn(T) -> bool
{
    let two = T::from(2).unwrap();
    let mut i = range.start;
    let mut j = range.end;
    while i < j {
        let h = i + (j - i) / two;
        if f(h) {
            j = h; // f(j) == true
        } else {
            i = h.succ(); // f(i-1) == false
        }
    }
    i
}

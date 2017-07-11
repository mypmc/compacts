#![feature(i128_type)]

pub trait UnsignedInt: PartialEq + PartialOrd + Eq + Ord + Copy {
    const WIDTH: usize;

    const MIN_BOUND: Self;
    const MAX_BOUND: Self;

    fn count_ones(self) -> u32;
    fn count_zeros(self) -> u32;

    fn succ(&self) -> Self;
    fn pred(&self) -> Self;
}

macro_rules! impl_UnsignedInt {
    ( $( ( $this:ty, $size:expr ) ),* ) => ($(
        impl UnsignedInt for $this {
            const WIDTH: usize = $size;

            const MIN_BOUND: Self = 0;
            const MAX_BOUND: Self = !(0 as Self);

            #[inline(always)] fn count_ones(self)  -> u32 {self.count_ones()}
            #[inline(always)] fn count_zeros(self) -> u32 {self.count_zeros()}

            #[inline(always)] fn succ(&self) -> Self {*self + 1}
            #[inline(always)] fn pred(&self) -> Self {*self - 1}
        }
    )*)
}

impl_UnsignedInt!((u64, 64), (u32, 32), (u16, 16), (u8, 8));

#[cfg(target_pointer_width = "32")]
impl_UnsignedInt!((usize, 32));
#[cfg(target_pointer_width = "64")]
impl_UnsignedInt!((usize, 64));

impl_UnsignedInt!((u128, 128));

/// Find the smallest index i in range at which f(i) is true, assuming that
/// f(i) == true implies f(i+1) == true.
#[macro_export]
macro_rules! search {
    ( $start:expr, $end:expr, $func:expr ) => {
        {
            let mut i = $start;
            let mut j = $end;
            while i < j {
                let h = i + (j - i) / 2;
                if $func(h) {
                    j = h; // f(j) == true
                } else {
                    i = h + 1; // f(i-1) == false
                }
            }
            i
        }
    }
}

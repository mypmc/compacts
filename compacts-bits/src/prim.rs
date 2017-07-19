pub trait UnsignedInt: PartialEq + PartialOrd + Eq + Ord + Copy + 'static {
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

            #[inline(always)] fn count_ones(self) -> u32 {
                self.count_ones()
            }

            #[inline(always)] fn count_zeros(self) -> u32 {
                self.count_zeros()
            }

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

pub(crate) trait Split {
    type Parts;
    fn split(&self) -> Self::Parts;
}

pub(crate) trait Merge {
    type Parts;
    fn merge(Self::Parts) -> Self;
}

macro_rules! impl_SplitMerge {
    ($( ( $this:ty, $half:ty ) ),*) => ($(
        impl Split for $this {
            type Parts = ($half, $half);
            #[inline] fn split(&self) -> Self::Parts {
                let this = *self;
                let s = Self::WIDTH / 2;
                ((this >> s) as $half, this as $half)
            }
        }
        impl Merge for $this {
            type Parts = ($half, $half);
            #[inline] fn merge(parts: Self::Parts) -> $this {
                let s = Self::WIDTH / 2;
                (parts.0 as $this << s) | parts.1 as $this
            }
        }
    )*)
}

impl_SplitMerge!((u64, u32), (u32, u16), (u16, u8));
#[cfg(target_pointer_width = "32")]
impl_SplitMerge!{(usize, u16)}
#[cfg(target_pointer_width = "64")]
impl_SplitMerge!{(usize, u32)}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn identity() {
        let w = 0b_11000010_11001000_u64;
        assert!(w == <u64 as Merge>::merge(w.split()));
        let w = 0b_10001011_10100100_u64;
        assert!(w == <u64 as Merge>::merge(w.split()));
    }
}

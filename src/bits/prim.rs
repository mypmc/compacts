pub(crate) trait Split {
    type Target;
    fn split(&self) -> (Self::Target, Self::Target);
}

pub(crate) trait Merge {
    type Source;
    fn merge((Self::Source, Self::Source)) -> Self;
}

macro_rules! impl_SplitMerge {
    ($( ( $this:ty, $half:ty, $size:expr ) ),*) => ($(
        impl Split for $this {
            type Target = $half;
            #[inline] fn split(&self) -> (Self::Target, Self::Target) {
                let this = *self;
                let s = $size / 2;
                ((this >> s) as $half, this as $half)
            }
        }
        impl Merge for $this {
            type Source = $half;

            #[inline]
            #[cfg_attr(feature = "cargo-clippy", allow(cast_lossless))]
            fn merge(parts: (Self::Source, Self::Source)) -> $this {
                let s = $size / 2;
                (parts.0 as $this << s) | parts.1 as $this
            }
        }
    )*)
}
impl_SplitMerge!((u64, u32, 64), (u32, u16, 32), (u16, u8, 16));

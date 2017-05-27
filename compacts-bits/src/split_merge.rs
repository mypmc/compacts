use UnsignedInt;

pub trait Split {
    type Parts;
    fn split(&self) -> Self::Parts;
}

pub trait Merge {
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

#[test]
fn split_merge() {
    let w = 0b_1100_u64;
    assert!(w == <u64 as Merge>::merge(w.split()));
}

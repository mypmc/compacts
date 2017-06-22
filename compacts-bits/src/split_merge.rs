use UnsignedInt;

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

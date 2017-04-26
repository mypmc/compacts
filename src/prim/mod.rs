pub use num::{Unsigned, PrimInt as Int, Bounded, One, Zero, NumCast as Cast};

pub trait Uint: Unsigned + Int {
    const WIDTH: usize;

    fn min_bound() -> Self {
        Self::min_value()
    }
    fn max_bound() -> Self {
        Self::max_value()
    }

    fn succ(&self) -> Self {
        *self + Self::one()
    }
    fn pred(&self) -> Self {
        *self - Self::one()
    }
    fn incr(&mut self) {
        *self = self.succ()
    }
    fn decr(&mut self) {
        *self = self.pred()
    }
}

macro_rules! impl_Uint {
    ( $( ( $this:ty, $size:expr ) ),* ) => ($(
        impl Uint for $this {
            const WIDTH: usize = $size;
        }
    )*)
}

impl_Uint!((u64, 64), (u32, 32), (u16, 16), (u8, 8));
#[cfg(target_pointer_width = "32")]
impl_Uint!((usize, 32));
#[cfg(target_pointer_width = "64")]
impl_Uint!((usize, 64));

pub static TRUE: &bool = &true;
pub static FALSE: &bool = &false;

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

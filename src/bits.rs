/// Constant sized bits.
pub trait Bits {
    /// Size of this representation.
    const SIZE: u64;

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

/// Utility trait for internal use.
pub trait Bounded {
    const MIN: Self;
    const MAX: Self;
}

/// Type for count bits Population
/// Prevent to use `u32` for `1 << 16`, or `u64` for `1 << 32`.
#[derive(Debug, Clone)]
pub enum PopCount<T: Bounded> {
    Ones(T),
    Full,
}

macro_rules! impl_Bits {
    ( $( ($type: ty, $size: expr) ),* ) => ($(
        impl Bounded for $type {
            const MIN: $type =  0;
            const MAX: $type = !0;
        }

        impl Bits for $type {
            const SIZE: u64 = $size;
            #[inline] fn ones(&self) -> u64 {
                let ones = self.count_ones();
                debug_assert!(ones as u64 <= Self::SIZE);
                ones as u64
            }
        }
    )*)
}
impl_Bits!((u64, 64), (u32, 32), (u16, 16), (u8, 8));
#[cfg(target_pointer_width = "32")]
impl_Bits!{(usize, 32)}
#[cfg(target_pointer_width = "64")]
impl_Bits!{(usize, 64)}

pub trait SplitMerge {
    type Parts;
    fn split(&self) -> Self::Parts;
    fn merge(Self::Parts) -> Self;
}

macro_rules! impl_SplitMerge {
    ($( ( $this:ty, $half:ty ) ),*) => ($(
        impl SplitMerge for $this {
            type Parts = ($half, $half);
            #[inline]
            fn split(&self) -> Self::Parts {
                let this = *self;
                let s = Self::SIZE / 2;
                ((this >> s) as $half, this as $half)
            }
            #[inline]
            fn merge(parts: Self::Parts) -> $this {
                let s = Self::SIZE / 2;
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

/*
impl<T, S> SplitMerge for S
    where S: From<(T, T)> + Into<(T, T)>
{
    type Parts = (T, T);
    fn split(self) -> Self::Parts {
        self.into()
    }
    fn merge(t: Self::Parts) -> S {
        Self::from(t)
    }
}
*/

macro_rules! impl_PopCount {
    ( $( $type: ty ),* ) => ($(
        impl Bounded for PopCount<$type> {
            const MIN: Self = PopCount::Ones(<$type as Bounded>::MIN);
            const MAX: Self = PopCount::Full;
        }

        impl PopCount<$type> {
            pub fn new(c: u64) -> PopCount<$type> {
                let max = <$type as Bounded>::MAX as u64 + 1;
                if max < c {
                    debug_assert!(false, "PopCount overflow");
                    PopCount::Full
                } else if max == c {
                    PopCount::Full
                } else {
                    PopCount::Ones(c as $type)
                }
            }
            pub fn count(&self) -> u64 {
                match self {
                    &PopCount::Ones(p) => p as u64,
                    &PopCount::Full    => <$type as Bounded>::MAX as u64 + 1,
                }
            }
            pub fn incr(&mut self) {
                let ones = self.count();
                match self {
                    this @ &mut PopCount::Ones(..) => {
                        if ones < <$type as Bounded>::MAX as u64 {
                            *this = PopCount::Ones(ones as $type + 1);
                        } else {
                            *this = PopCount::Full;
                        }
                    },
                    &mut PopCount::Full => {
                        debug_assert!(false, "PopCount overflow");
                    }
                }
            }
            pub fn decr(&mut self) {
                let ones = self.count();
                match self {
                    this @ &mut PopCount::Ones(..) => {
                        if ones > <$type as Bounded>::MIN as u64 {
                            *this = PopCount::Ones(ones as $type - 1);
                        } else {
                            debug_assert!(false, "PopCount overflow");
                        }
                    },
                    this @ &mut PopCount::Full => {
                        *this = PopCount::Ones(<$type as Bounded>::MAX);
                    }
                }
            }
        }
    )*);
}

impl_PopCount!(u32, u16, u8);

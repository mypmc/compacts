/// Constant sized bits.
pub trait Bits {
    /// Max bits size of this representation.
    /// ones() + zeros() == CAPACITY
    const CAPACITY: u64;

    /// Count non-zero bits.
    // REQUIRES: ones() <= Self::CAPACITY
    fn ones(&self) -> u64 {
        Self::CAPACITY - self.zeros()
    }

    /// Count zero bits.
    // REQUIRES: zeros() <= Self::CAPACITY
    fn zeros(&self) -> u64 {
        Self::CAPACITY - self.ones()
    }
}

/// Utility trait for internal use.
pub trait Bounded {
    const MIN: Self;
    const MAX: Self;
}

/// Type prevent to use `u32` for `1 << 16`, or `u64` for `1 << 32`
#[derive(Debug, Clone)]
pub enum Count<T: Bounded> {
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
            const CAPACITY: u64 = $size;
            #[inline] fn ones(&self) -> u64 {
                let ones = self.count_ones() as u64;
                debug_assert!(ones <= Self::CAPACITY);
                ones
            }
        }
    )*)
}
impl_Bits!((u64, 64), (u32, 32), (u16, 16), (u8, 8));
#[cfg(target_pointer_width = "32")]
impl_Bits!{(usize, 32)}
#[cfg(target_pointer_width = "64")]
impl_Bits!{(usize, 64)}

macro_rules! impl_Count {
    ( $( $type: ty ),* ) => ($(
        impl Bounded for Count<$type> {
            const MIN: Self = Count::Ones(<$type as Bounded>::MIN);
            const MAX: Self = Count::Full;
        }

        impl Count<$type> {
            pub fn new(c: u64) -> Count<$type> {
                let max = <$type as Bounded>::MAX as u64 + 1;
                if max <= c {
                    Count::Full
                } else {
                    Count::Ones(c as $type)
                }
            }
            pub fn count(&self) -> u64 {
                match self {
                    &Count::Ones(p) => p as u64,
                    &Count::Full    => <$type as Bounded>::MAX as u64 + 1,
                }
            }
            pub fn incr(&mut self) {
                let ones = self.count();
                match self {
                    this @ &mut Count::Ones(..) => {
                        if ones < <$type as Bounded>::MAX as u64 {
                            *this = Count::Ones(ones as $type + 1);
                        } else {
                            *this = Count::Full;
                        }
                    },
                    &mut Count::Full => {
                        debug_assert!(false, "increment overflow");
                    }
                }
            }
            pub fn decr(&mut self) {
                let ones = self.count();
                match self {
                    this @ &mut Count::Ones(..) => {
                        if ones > <$type as Bounded>::MIN as u64 {
                            *this = Count::Ones(ones as $type - 1);
                        } else {
                            debug_assert!(false, "decrement overflow");
                        }
                    },
                    this @ &mut Count::Full => {
                        *this = Count::Ones(<$type as Bounded>::MAX);
                    }
                }
            }
        }
    )*);
}

impl_Count!(u32, u16, u8);

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
                let s = Self::CAPACITY / 2;
                ((this >> s) as $half, this as $half)
            }
            #[inline]
            fn merge(parts: Self::Parts) -> $this {
                let s = Self::CAPACITY / 2;
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

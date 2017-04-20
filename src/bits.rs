use {Bounded, PopCount};

/// Type prevent to use `u32` for `1 << 16`, or `u64` for `1 << 32`
#[derive(Debug, Clone)]
pub enum Count<T: Bounded> {
    Ones(T),
    Full,
}

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
            pub fn value(&self) -> u64 {
                match self {
                    &Count::Ones(p) => p as u64,
                    &Count::Full    => <$type as Bounded>::MAX as u64 + 1,
                }
            }
            pub fn incr(&mut self) {
                let ones = self.value();
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
                let ones = self.value();
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

#[test]
fn split_merge() {
    let w = 0b_1100_u64;
    assert!(w == <u64 as SplitMerge>::merge(w.split()));
}

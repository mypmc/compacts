use std::fmt;

/// Helper trait and struct for internal use.

pub trait Bounded: Sized {
    const MIN: Self;
    const MAX: Self;
}

/// We need to use `u32` for `u16::MAX + 1`, or `u64` for `u32::MAX + 1`.
/// PopCount can handle this.
#[derive(Clone)]
pub enum PopCount<T: Bounded> {
    Ones(T),
    Full,
}

macro_rules! implPopCount {
    ( $( $type: ty ),* ) => ($(
        impl Bounded for $type {
            const MIN: $type =  0;
            const MAX: $type = !0; //::std::$mod::MAX;
        }

        impl fmt::Debug for PopCount<$type> {
            fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                write!(fmt, "PopCount({})", self.ones())
            }
        }
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
            pub fn ones(&self) -> u64 {
                match self {
                    &PopCount::Ones(p) => p as u64,
                    &PopCount::Full    => <$type as Bounded>::MAX as u64 + 1,
                }
            }
            pub fn incr(&mut self) {
                let ones = self.ones();
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
                let ones = self.ones();
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

implPopCount!(u32, u16, u8);

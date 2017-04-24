//! Internal representaions of fixed size bit storage.

use std::{fmt, u16};

use {bits, PopCount, Bounded};

macro_rules! keypos {
    ( $bit: expr, $key: ident, $pos: ident ) => (
        let key  = $bit / <u64 as PopCount>::CAPACITY as u16;
        let $pos = $bit % <u64 as PopCount>::CAPACITY as u16;
        let $key = key as usize;
    );
}
macro_rules! bitmask {
    ( $bit: expr, $key: ident, $mask: ident ) => (
        keypos!($bit, $key, pos);
        let $mask = 1 << pos;
    );
}
macro_rules! pair {
    ( $op:ident, $iterable1:expr, $iterable2:expr ) => {
        pair::$op($iterable1.iter(), $iterable2.iter())
    };
}

mod iter;
pub use self::iter::{Iter, IntoIter, Pointer};

mod pair;
mod bitand;
mod bitor;
mod bitxor;

mod rank;
mod select;

#[cfg(test)]
mod tests;

//#[derive(Clone)]
pub enum Bucket {
    // Vec holds bit as is, with sorted order.
    Vec(bits::Count<u16>, Vec<u16>),
    // Each elements represents bit-array.
    Map(bits::Count<u16>, Box<[u64; Bucket::MAP_CAPACITY]>),
}

impl Clone for Bucket {
    fn clone(&self) -> Self {
        match self {
            &Bucket::Vec(ref pop, ref vec) => Bucket::Vec(pop.clone(), vec.clone()),
            &Bucket::Map(ref pop, box map) => Bucket::Map(pop.clone(), Box::new(map)),
        }
    }
}
impl fmt::Debug for Bucket {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Bucket::Vec(ref pop, _) => write!(fmt, "Vec({:?})", pop.value()),
            &Bucket::Map(ref pop, _) => write!(fmt, "Map({:?})", pop.value()),
        }
    }
}

impl PopCount for Bucket {
    const CAPACITY: u64 = 1 << 16;

    fn ones(&self) -> u64 {
        match self {
            &Bucket::Vec(ref pop, _) => pop.value(),
            &Bucket::Map(ref pop, _) => pop.value(),
        }
    }
}

impl Bucket {
    const BITS_SIZE: usize = <u64 as PopCount>::CAPACITY as usize;

    const VEC_CAPACITY: usize = 1024; // 1 << 10
    const MAP_CAPACITY: usize = 1024;

    #[allow(dead_code)]
    fn load_factor(&self) -> f64 {
        self.ones() as f64 / Self::CAPACITY as f64
    }

    pub fn new() -> Bucket {
        Bucket::Vec(bits::Count::MIN, Vec::new())
    }
    pub fn with_capacity(cap: usize) -> Bucket {
        if cap <= Bucket::VEC_CAPACITY {
            let vec = Vec::with_capacity(cap);
            Bucket::Vec(bits::Count::MIN, vec)
        } else {
            let arr = Box::new([0; Bucket::MAP_CAPACITY]);
            Bucket::Map(bits::Count::MIN, arr)
        }
    }

    /// Convert to more size efficient bits representaions.
    pub fn optimize(&mut self) {
        if self.fitted() {
            self.shrink();
            return;
        }
        *self = self.iter().collect::<Bucket>();
    }
    fn fitted(&mut self) -> bool {
        let ones = self.ones();
        let max = Self::VEC_CAPACITY as u64;
        match self {
            &mut Bucket::Vec(..) if ones > max => false,
            &mut Bucket::Map(..) if ones <= max => false,
            _ => true,
        }
    }
    fn shrink(&mut self) {
        match self {
            &mut Bucket::Vec(_, ref mut bits) => bits.shrink_to_fit(),
            &mut Bucket::Map(..) => { /* ignore */ }
        }
    }
}

impl Bucket {
    pub fn contains(&self, bit: u16) -> bool {
        match self {
            &Bucket::Vec(_, ref bits) => bits.binary_search(&bit).is_ok(),
            &Bucket::Map(_, ref bits) => {
                bitmask!(bit, key, mask);
                bits.get(key).map_or(false, |map| *map & mask != 0)
            }
        }
    }

    pub fn insert(&mut self, bit: u16) -> bool {
        match self {
            &mut Bucket::Vec(ref mut popc, ref mut bits) => {
                let ok = bits.binary_search(&bit)
                    .map_err(|i| bits.insert(i, bit))
                    .is_err();
                if ok {
                    popc.incr();
                }
                ok
            }
            &mut Bucket::Map(ref mut popc, ref mut bits) => {
                bitmask!(bit, key, mask);
                if bits[key] & mask != 0 {
                    false
                } else {
                    bits[key] |= mask;
                    popc.incr();
                    true
                }
            }
        }
    }

    pub fn remove(&mut self, bit: u16) -> bool {
        match self {
            &mut Bucket::Vec(ref mut popc, ref mut bits) => {
                let ok = bits.binary_search(&bit)
                    .map(|i| {
                             let removed = bits.remove(i);
                             debug_assert_eq!(bit, removed);
                         })
                    .is_ok();
                if ok {
                    popc.decr();
                }
                ok
            }
            &mut Bucket::Map(ref mut popc, ref mut bits) => {
                bitmask!(bit, key, mask);
                if bits[key] & mask != 0 {
                    bits[key] &= !mask;
                    popc.decr();
                    true
                } else {
                    false
                }
            }
        }
    }
}

//! Internal representaions of fixed size bit storage.

use std::{fmt, u16};
use std::iter::{IntoIterator, FromIterator};

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
pub use self::iter::Iter;

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
    // Cow<[u64]>
    Map(bits::Count<u16>, Box<[u64; Bucket::MAP_CAPACITY]>),
}

impl fmt::Debug for Bucket {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Bucket::Vec(ref pop, _) => write!(fmt, "Vec({:?})", pop.value()),
            &Bucket::Map(ref pop, _) => write!(fmt, "Map({:?})", pop.value()),
        }
    }
}
impl Clone for Bucket {
    fn clone(&self) -> Self {
        match self {
            &Bucket::Vec(ref pop, ref vec) => Bucket::Vec(pop.clone(), vec.clone()),
            &Bucket::Map(ref pop, box map) => Bucket::Map(pop.clone(), Box::new(map)),
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
    pub fn iter(&self) -> Iter {
        match self {
            &Bucket::Vec(ref pop, ref bits) => Iter::vec(&bits[..], pop),
            &Bucket::Map(ref pop, ref bits) => Iter::map(&bits[..], pop),
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

impl FromIterator<u16> for Bucket {
    fn from_iter<I: IntoIterator<Item = u16>>(iterable: I) -> Self {
        let iter = iterable.into_iter();
        let (min, maybe) = iter.size_hint();
        let mut repr = Bucket::with_capacity(if let Some(max) = maybe { max } else { min });
        let ones = insert_u16_all(iter, &mut repr);
        debug_assert_eq!(ones, repr.ones());
        repr
    }
}
impl<'a> FromIterator<&'a u16> for Bucket {
    fn from_iter<I: IntoIterator<Item = &'a u16>>(iterable: I) -> Self {
        let iter = iterable.into_iter();
        iter.cloned().collect::<Bucket>()
    }
}
impl FromIterator<bool> for Bucket {
    fn from_iter<I: IntoIterator<Item = bool>>(iterable: I) -> Bucket {
        let iter = iterable.into_iter();
        iter.take(Bucket::CAPACITY as usize)
            .enumerate()
            .filter_map(|(i, p)| if p { Some(i as u16) } else { None })
            .collect::<Bucket>()
    }
}
impl<'a> FromIterator<&'a bool> for Bucket {
    fn from_iter<I: IntoIterator<Item = &'a bool>>(iterable: I) -> Bucket {
        let iter = iterable.into_iter();
        iter.cloned().collect::<Bucket>()
    }
}

fn insert_u16_all<It: Iterator<Item = u16>>(it: It, repr: &mut Bucket) -> u64 {
    let mut ones = 0;
    for item in it {
        if repr.insert(item) {
            ones += 1;
        }
    }
    ones
}

impl<'a> IntoIterator for &'a Bucket {
    type Item = <Iter<'a> as Iterator>::Item;
    type IntoIter = Iter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
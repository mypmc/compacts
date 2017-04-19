//! Internal representaions of fixed size bit storage.

use std::{fmt, u16};
use std::iter::{IntoIterator, FromIterator};

use super::{Bits, Bounded, Count};
use super::{Rank0, Rank1, Select0, Select1};

macro_rules! keypos {
    ( $bit: expr, $key: ident, $pos: ident ) => (
        // 64 == Bucket::BITS_SIZE
        let key  = $bit / 64;
        let $pos = $bit % 64;
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

#[derive(Clone)]
pub enum Bucket {
    // Vec hold bit as is, sorted order.
    Vec(Count<u16>, Vec<u16>),

    // Map hold u64 as a bitarray, each non-zero bit represents element.
    Map(Count<u16>, Vec<u64>),
}
impl Bits for Bucket {
    const SIZE: u64 = 1 << 16;

    fn ones(&self) -> u64 {
        match self {
            &Bucket::Vec(ref pop, _) => pop.count(),
            &Bucket::Map(ref pop, _) => pop.count(),
        }
    }
}

impl Bucket {
    const BITS_SIZE: u64 = <u64 as Bits>::SIZE;

    //pub const VEC_SIZE: u64 = 1 << 12;
    //pub const VEC_SIZE: u64 = 1 << 11;
    const VEC_SIZE: u64 = 1 << 10;

    #[allow(dead_code)]
    const MAP_SIZE: u64 = Bucket::SIZE / Bucket::BITS_SIZE;

    #[allow(dead_code)]
    fn load_factor(&self) -> f64 {
        self.ones() as f64 / Self::SIZE as f64
    }

    pub fn new() -> Bucket {
        Bucket::Vec(Count::MIN, Vec::new())
    }
    pub fn with_capacity(cap: usize) -> Bucket {
        if cap as u64 <= Self::VEC_SIZE {
            Bucket::Vec(Count::MIN, Vec::with_capacity(cap))
        } else {
            Bucket::Map(Count::MIN, Vec::with_capacity(cap))
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
        match self {
            &mut Bucket::Vec(..) if ones > Self::VEC_SIZE => false,
            &mut Bucket::Map(..) if ones <= Self::VEC_SIZE => false,
            _ => true,
        }
    }
    fn shrink(&mut self) {
        match self {
            &mut Bucket::Vec(_, ref mut bits) => bits.shrink_to_fit(),
            &mut Bucket::Map(_, ref mut bits) => bits.shrink_to_fit(),
        }
    }
}

impl Bucket {
    fn iter(&self) -> Iter {
        match self {
            &Bucket::Vec(ref pop, ref bits) => Iter::vec(&bits[..], pop),
            &Bucket::Map(ref pop, ref bits) => Iter::map(&bits[..], pop),
        }
    }
}

impl fmt::Debug for Bucket {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Bucket::Vec(ref pop, _) => write!(fmt, "Vec({:?})", pop.count()),
            &Bucket::Map(ref pop, _) => write!(fmt, "Map({:?})", pop.count()),
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
                if let Some(map) = bits.get_mut(key) {
                    if *map & mask != 0 {
                        return false;
                    } else {
                        *map |= mask;
                        popc.incr();
                        return true;
                    }
                }
                if key > bits.len() {
                    bits.resize(key, 0);
                }
                bits.insert(key, mask);
                popc.incr();
                return true;
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
                if let Some(map) = bits.get_mut(key) {
                    if *map & mask != 0 {
                        *map &= !mask;
                        popc.decr();
                        return true;
                    } else {
                        return false;
                    };
                }
                return false;
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
        iter.take(Bucket::SIZE as usize)
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

//! Internal representaions of fixed size bit storage.

use std::{fmt, u16};
use std::iter::{IntoIterator, FromIterator};

use super::{Bits, Bounded, PopCount};
use super::{Rank0, Rank1, Select0, Select1};

macro_rules! keypos {
    ( $bit: expr, $key: ident, $pos: ident ) => (
        // 64 == Repr::BITS_SIZE
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
pub enum Repr {
    // Vec hold bit as is, sorted order.
    Vec(PopCount<u16>, Vec<u16>),

    // Map hold u64 as a bitarray, each non-zero bit represents element.
    Map(PopCount<u16>, Vec<u64>),
}
impl Bits for Repr {
    const SIZE: u64 = 1 << 16;

    fn none() -> Self {
        Self::new()
    }
    fn ones(&self) -> u64 {
        match self {
            &Repr::Vec(ref pop, _) => pop.ones(),
            &Repr::Map(ref pop, _) => pop.ones(),
        }
    }
}

impl Repr {
    pub const BITS_SIZE: u64 = <u64 as Bits>::SIZE;

    //pub const VEC_SIZE: u64 = 1 << 12;
    //pub const VEC_SIZE: u64 = 1 << 11;
    const VEC_SIZE: u64 = 1 << 10;

    #[allow(dead_code)]
    const MAP_SIZE: u64 = Repr::SIZE / Repr::BITS_SIZE;

    #[allow(dead_code)]
    fn load_factor(&self) -> f64 {
        self.ones() as f64 / Self::SIZE as f64
    }

    pub fn new() -> Repr {
        Repr::Vec(PopCount::MIN, Vec::new())
    }
    pub fn with_capacity(cap: usize) -> Repr {
        if cap as u64 <= Self::VEC_SIZE {
            Repr::Vec(PopCount::MIN, Vec::with_capacity(cap))
        } else {
            Repr::Map(PopCount::MIN, Vec::with_capacity(cap))
        }
    }

    /// Convert to more size efficient bits representaions.
    pub fn optimize(&mut self) {
        if self.fitted() {
            self.shrink();
            return;
        }
        *self = self.iter().collect::<Repr>();
    }
    fn fitted(&mut self) -> bool {
        let ones = self.ones();
        match self {
            &mut Repr::Vec(..) if ones > Self::VEC_SIZE => false,
            &mut Repr::Map(..) if ones <= Self::VEC_SIZE => false,
            _ => true,
        }
    }
    fn shrink(&mut self) {
        match self {
            &mut Repr::Vec(_, ref mut bits) => bits.shrink_to_fit(),
            &mut Repr::Map(_, ref mut bits) => bits.shrink_to_fit(),
        }
    }
}

impl Repr {
    fn iter(&self) -> Iter {
        match self {
            &Repr::Vec(ref pop, ref bits) => Iter::vec(&bits[..], pop.ones() as usize),
            &Repr::Map(ref pop, ref bits) => Iter::map(&bits[..], pop.ones() as usize),
        }
    }
}

impl fmt::Debug for Repr {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Repr::Vec(ref popc, _) => write!(fmt, "Vec({:?})", popc.ones()),
            &Repr::Map(ref popc, _) => write!(fmt, "Map({:?})", popc.ones()),
        }
    }
}

impl Repr {
    pub fn contains(&self, bit: u16) -> bool {
        match self {
            &Repr::Vec(_, ref bits) => bits.binary_search(&bit).is_ok(),
            &Repr::Map(_, ref bits) => {
                bitmask!(bit, key, mask);
                bits.get(key).map_or(false, |map| *map & mask != 0)
            }
        }
    }

    pub fn insert(&mut self, bit: u16) -> bool {
        match self {
            &mut Repr::Vec(ref mut popc, ref mut bits) => {
                let ok = bits.binary_search(&bit)
                    .map_err(|i| bits.insert(i, bit))
                    .is_err();
                if ok {
                    popc.incr();
                }
                ok
            }
            &mut Repr::Map(ref mut popc, ref mut bits) => {
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
            &mut Repr::Vec(ref mut popc, ref mut bits) => {
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
            &mut Repr::Map(ref mut popc, ref mut bits) => {
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

impl FromIterator<u16> for Repr {
    fn from_iter<I: IntoIterator<Item = u16>>(iterable: I) -> Self {
        let iter = iterable.into_iter();
        let (min, maybe) = iter.size_hint();
        let mut repr = Repr::with_capacity(if let Some(max) = maybe { max } else { min });
        let ones = insert_u16_all(iter, &mut repr);
        debug_assert_eq!(ones, repr.ones());
        repr
    }
}
impl<'a> FromIterator<&'a u16> for Repr {
    fn from_iter<I: IntoIterator<Item = &'a u16>>(iterable: I) -> Self {
        let iter = iterable.into_iter();
        iter.cloned().collect::<Repr>()
    }
}
impl FromIterator<bool> for Repr {
    fn from_iter<I: IntoIterator<Item = bool>>(iterable: I) -> Repr {
        let iter = iterable.into_iter();
        iter.take(Repr::SIZE as usize)
            .enumerate()
            .filter_map(|(i, p)| if p { Some(i as u16) } else { None })
            .collect::<Repr>()
    }
}
impl<'a> FromIterator<&'a bool> for Repr {
    fn from_iter<I: IntoIterator<Item = &'a bool>>(iterable: I) -> Repr {
        let iter = iterable.into_iter();
        iter.cloned().collect::<Repr>()
    }
}

fn insert_u16_all<It: Iterator<Item = u16>>(it: It, repr: &mut Repr) -> u64 {
    let mut ones = 0;
    for item in it {
        if repr.insert(item) {
            ones += 1;
        }
    }
    ones
}

impl<'a> IntoIterator for &'a Repr {
    type Item = <Iter<'a> as Iterator>::Item;
    type IntoIter = Iter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

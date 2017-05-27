#[macro_use]
mod macros;

mod inner;
// mod rle16;

mod rank_select;
mod pairwise;

#[cfg(test)]
mod tests;

use std::fmt;

#[derive(Clone)]
pub enum Block {
    Vec16(inner::Bucket<u16>),
    Vec64(inner::Bucket<u64>),
    // Rle16(rle16::Bucket),
}
use self::Block::*;

impl fmt::Debug for Block {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let lf = self.load_factor();
        match *self {
            Vec16(..) => write!(fmt, "Vec16({:.3})", lf),
            Vec64(..) => write!(fmt, "Vec64({:.3})", lf),
        }
    }
}

impl Default for Block {
    fn default() -> Self {
        Vec16(inner::Bucket::<u16>::new())
    }
}

const VEC16_THRESHOLD: usize = 4096; // 4096 * 16 == 65536
// const VEC64_THRESHOLD: usize = 1024; // 1024 * 64 == 65536

impl Block {
    pub const CAPACITY: u32 = 1 << 16;

    pub fn new() -> Self {
        Block::default()
    }

    pub fn with_capacity(weight: usize) -> Self {
        if weight <= VEC16_THRESHOLD {
            Vec16(inner::Bucket::with_capacity(weight))
        } else {
            Vec64(inner::Bucket::<u64>::new())
        }
    }

    pub fn load_factor(&self) -> f64 {
        self.count_ones() as f64 / Self::CAPACITY as f64
    }

    pub fn count_ones(&self) -> u32 {
        match *self {
            Vec16(ref data) => data.weight,
            Vec64(ref data) => data.weight,
        }
    }

    pub fn count_zeros(&self) -> u32 {
        Self::CAPACITY - self.count_ones()
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn is_sorted(&self) -> bool {
        match *self {
            Vec16(..) => true,
            _ => false,
        }
    }
    pub fn is_mapped(&self) -> bool {
        match *self {
            Vec64(..) => true,
            _ => false,
        }
    }

    pub fn as_sorted(&mut self) {
        if !self.is_sorted() {
            *self = match *self {
                Vec64(ref b) => Vec16(inner::Bucket::<u16>::from(b)),
                _ => unreachable!(),
            };
        }
    }

    pub fn as_mapped(&mut self) {
        if !self.is_mapped() {
            *self = match *self {
                Vec16(ref b) => Vec64(inner::Bucket::<u64>::from(b)),
                _ => unreachable!(),
            }
        }
    }

    /// May convert to more efficient block representaions.
    pub fn optimize(&mut self) {
        let ones = self.count_ones();
        if ones == 0 {
            self.clear();
        }
        let max = <inner::Bucket<u16>>::THRESHOLD as u32;
        match *self {
            ref mut this @ Vec16(..) if ones > max => this.as_mapped(),
            ref mut this @ Vec64(..) if ones <= max => this.as_sorted(),
            _ => { /* ignore */ }
        }
    }
}

#[cfg_attr(rustfmt, rustfmt_skip)]
impl Block {
    pub fn contains(&self, bit: u16) -> bool   {delegate!(ref self, contains, bit)}
    pub fn insert(&mut self, bit: u16) -> bool {delegate!(ref mut self, insert, bit)}
    pub fn remove(&mut self, bit: u16) -> bool {delegate!(ref mut self, remove, bit)}
    pub fn iter(&self) -> inner::Iter          {delegate!(ref self, iter)}
}

impl ::std::ops::Index<u16> for Block {
    type Output = bool;
    fn index(&self, i: u16) -> &Self::Output {
        if self.contains(i) { ::TRUE } else { ::FALSE }
    }
}

impl<'a> ::std::iter::IntoIterator for &'a Block {
    type Item = <inner::Iter<'a> as Iterator>::Item;
    type IntoIter = inner::Iter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        delegate!(ref self, iter)
    }
}
impl ::std::iter::IntoIterator for Block {
    type Item = <inner::IntoIter as Iterator>::Item;
    type IntoIter = inner::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        delegate!(self, into_iter)
    }
}

impl ::std::iter::Extend<u16> for Block {
    fn extend<I>(&mut self, iterable: I)
        where I: ::std::iter::IntoIterator<Item = u16>
    {
        extend_by_u16!(self, iterable);
    }
}

impl ::std::iter::FromIterator<u16> for Block {
    fn from_iter<I>(iterable: I) -> Self
        where I: ::std::iter::IntoIterator<Item = u16>
    {
        let iter = iterable.into_iter();
        let (min, maybe) = iter.size_hint();
        let mut block = Block::with_capacity(if let Some(max) = maybe { max } else { min });
        let ones = extend_by_u16!(&mut block, iter);
        debug_assert_eq!(ones, block.count_ones());
        block
    }
}
impl<'a> ::std::iter::FromIterator<&'a u16> for Block {
    fn from_iter<I>(iterable: I) -> Self
        where I: ::std::iter::IntoIterator<Item = &'a u16>
    {
        let iter = iterable.into_iter();
        iter.cloned().collect::<Self>()
    }
}

impl ::std::iter::FromIterator<bool> for Block {
    fn from_iter<I>(iterable: I) -> Self
        where I: ::std::iter::IntoIterator<Item = bool>
    {
        let iter = iterable.into_iter();
        iter.take(Self::CAPACITY as usize)
            .enumerate()
            .filter_map(|(i, p)| if p { Some(i as u16) } else { None })
            .collect::<Self>()
    }
}
impl<'a> ::std::iter::FromIterator<&'a bool> for Block {
    fn from_iter<I>(iterable: I) -> Self
        where I: ::std::iter::IntoIterator<Item = &'a bool>
    {
        let iter = iterable.into_iter();
        iter.cloned().collect::<Self>()
    }
}

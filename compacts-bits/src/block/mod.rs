#[macro_use]
mod macros;

pub mod inner;

mod rank_select;
mod pairwise;

#[cfg(test)]
mod tests;

use std::fmt;

#[derive(Clone)]
pub enum Block {
    Vec16(inner::Seq16),
    Vec64(inner::Seq64),
    // Rle16(inner::Rle16),
}
use self::Block::*;

impl fmt::Debug for Block {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Vec16(ref d) => write!(fmt, "{:?}", d),
            Vec64(ref d) => write!(fmt, "{:?}", d),
            // Rle16(..) => write!(fmt, "Rle16({:.3})", lf),
        }
    }
}

impl Default for Block {
    fn default() -> Self {
        // default is seq64.
        Vec64(inner::Seq64::new())
    }
}

// const VEC16_THRESHOLD: usize = 4096; // 4096 * 16 == 65536
// const VEC64_THRESHOLD: usize = 1024; // 1024 * 64 == 65536

impl Block {
    pub const CAPACITY: u32 = 1 << 16;

    pub fn new() -> Self {
        Block::default()
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn as_vec16(&mut self) {
        *self = match *self {
            Vec64(ref b) => Vec16(inner::Seq16::from(b)),
            // Rle16(ref b) => Vec16(inner::Seq16::from(b)),
            _ => unreachable!(),
        }
    }

    pub fn as_vec64(&mut self) {
        *self = match *self {
            Vec16(ref b) => Vec64(inner::Seq64::from(b)),
            // Rle16(ref b) => Vec64(inner::Seq64::from(b)),
            _ => unreachable!(),
        }
    }

    // pub fn as_rle16(&mut self) {
    //     *self = match *self {
    //         Vec16(ref b) => Rle16(inner::Rle16::from(b)),
    //         Vec64(ref b) => Rle16(inner::Rle16::from(b)),
    //         _ => { /* ignore */ }
    //     }
    // }

    /// May convert to more efficient block representaions.
    /// This may consume many time and resource. So, don't call too much.
    pub fn optimize(&mut self) {
        let ones = self.count_ones();
        if ones == 0 {
            self.clear();
        }
        let max = <inner::Seq16>::THRESHOLD as u32;
        match *self {
            ref mut this @ Vec16(..) if ones > max => this.as_vec64(),
            ref mut this @ Vec64(..) if ones <= max => this.as_vec16(),
            _ => { /* ignore */ }
        }
    }
}

#[cfg_attr(rustfmt, rustfmt_skip)]
impl Block {
    pub fn count_ones(&self)  -> u32 {delegate!(ref self, count_ones)}
    pub fn count_zeros(&self) -> u32 {delegate!(ref self, count_zeros)}
    pub fn load_factor(&self) -> f64 {delegate!(ref self, load_factor)}

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
        let mut block = Block::new();
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

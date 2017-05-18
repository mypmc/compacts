//! Internal representaions of fixed size bit storage.

use std::{fmt, u16};
use std::iter::{FromIterator, IntoIterator, Extend};
use std::ops::Index;

mod bucket;
mod impl_pairwise;
mod impl_ranked;

#[cfg(test)]
mod tests;

use self::bucket::Bucket;
pub use self::bucket::{Iter, IntoIter};

use dict::prim::{self, Uint};
use dict::Ranked;

const U64_WIDTH: usize = <u64 as Uint>::WIDTH;
const THRESHOLD: usize = <Bucket<u16>>::THRESHOLD;

#[derive(Clone)]
pub enum Block {
    // Holds bit as is, with sorted order.
    Sorted(Bucket<u16>),
    // Each elements represents bit-array that length is 64.
    Mapped(Bucket<u64>),
}

impl Block {
    pub const CAPACITY: u32 = 1 << 16; // same type with Self::Weight

    fn load_factor(&self) -> f64 {
        self.count1() as f64 / Self::CAPACITY as f64
    }
}


impl fmt::Debug for Block {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let lf = self.load_factor();
        match *self {
            Block::Sorted(..) => write!(fmt, "Sorted({:.3})", lf),
            Block::Mapped(..) => write!(fmt, "Mapped({:.3})", lf),
        }
    }
}

impl Index<u16> for Block {
    type Output = bool;
    fn index(&self, i: u16) -> &Self::Output {
        if self.contains(i) {
            prim::TRUE
        } else {
            prim::FALSE
        }
    }
}

impl Default for Block {
    fn default() -> Self {
        Block::Sorted(Bucket::<u16>::new())
    }
}

impl Block {
    pub fn new() -> Self {
        Block::default()
    }
    pub fn with_capacity(w: usize) -> Self {
        if w <= THRESHOLD {
            Block::Sorted(Bucket::with_capacity(w))
        } else {
            Block::Mapped(Bucket::<u64>::new())
        }
    }
}

impl Block {
    pub fn is_sorted(&self) -> bool {
        match *self {
            Block::Sorted(..) => true,
            _ => false,
        }
    }
    pub fn is_mapped(&self) -> bool {
        match *self {
            Block::Mapped(..) => true,
            _ => false,
        }
    }

    pub fn as_sorted(&mut self) {
        if !self.is_sorted() {
            *self = match *self {
                Block::Mapped(ref b) => Block::Sorted(Bucket::<u16>::from(b)),
                _ => unreachable!(),
            };
        }
    }

    pub fn as_mapped(&mut self) {
        if !self.is_mapped() {
            *self = match *self {
                Block::Sorted(ref b) => Block::Mapped(Bucket::<u64>::from(b)),
                _ => unreachable!(),
            }
        }
    }

    pub fn clear(&mut self) {
        match self {
            this @ &mut Block::Sorted(..) => *this = Block::Sorted(Bucket::<u16>::new()),
            this @ &mut Block::Mapped(..) => *this = Block::Mapped(Bucket::<u64>::new()),
        }
    }

    /// May convert to more efficient block representaions.
    pub fn optimize(&mut self) {
        let ones = self.count1();
        if ones == 0 {
            self.clear();
        }
        let max = THRESHOLD as u32;
        match *self {
            ref mut this @ Block::Sorted(..) if ones > max => this.as_mapped(),
            ref mut this @ Block::Mapped(..) if ones <= max => this.as_sorted(),
            _ => { /* ignore */ }
        }
    }
}

#[cfg_attr(rustfmt, rustfmt_skip)]
impl Block {
    pub fn contains(&self, bit: u16) -> bool   {delegate!(ref self, contains, bit)}
    pub fn insert(&mut self, bit: u16) -> bool {delegate!(ref mut self, insert, bit)}
    pub fn remove(&mut self, bit: u16) -> bool {delegate!(ref mut self, remove, bit)}
    pub fn iter(&self) -> Iter                 {delegate!(ref self, iter)}
    pub fn into_iter(self) -> IntoIter         {delegate!(self, into_iter)}
}

impl<'a> IntoIterator for &'a Block {
    type Item = <Iter<'a> as Iterator>::Item;
    type IntoIter = Iter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl IntoIterator for Block {
    type Item = <IntoIter as Iterator>::Item;
    type IntoIter = IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.into_iter()
    }
}

impl Extend<u16> for Block {
    fn extend<I>(&mut self, iterable: I)
        where I: IntoIterator<Item = u16>
    {
        extend_by_u16!(self, iterable);
    }
}

impl FromIterator<u16> for Block {
    fn from_iter<I>(iterable: I) -> Self
        where I: IntoIterator<Item = u16>
    {
        let iter = iterable.into_iter();
        let (min, maybe) = iter.size_hint();
        let mut block = Block::with_capacity(if let Some(max) = maybe { max } else { min });
        let ones = extend_by_u16!(&mut block, iter);
        debug_assert_eq!(ones, block.count1());
        block
    }
}
impl<'a> FromIterator<&'a u16> for Block {
    fn from_iter<I>(iterable: I) -> Self
        where I: IntoIterator<Item = &'a u16>
    {
        let iter = iterable.into_iter();
        iter.cloned().collect::<Self>()
    }
}

impl FromIterator<bool> for Block {
    fn from_iter<I>(iterable: I) -> Self
        where I: IntoIterator<Item = bool>
    {
        let iter = iterable.into_iter();
        iter.take(Block::CAPACITY as usize)
            .enumerate()
            .filter_map(|(i, p)| if p { Some(i as u16) } else { None })
            .collect::<Block>()
    }
}
impl<'a> FromIterator<&'a bool> for Block {
    fn from_iter<I>(iterable: I) -> Self
        where I: IntoIterator<Item = &'a bool>
    {
        let iter = iterable.into_iter();
        iter.cloned().collect::<Self>()
    }
}

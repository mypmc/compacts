use std::iter::{Cloned, ExactSizeIterator};
use std::slice::Iter as SliceIter;
use std::borrow::Cow;

use super::{Seq16, Seq64, Rle16};

pub enum Iter<'a> {
    U16(ClonedIter<'a, u16>),
    U64(PackedIter<'a>),
    Run(RunIter<'a>),
}

type ClonedIter<'a, T> = Cloned<SliceIter<'a, T>>;

pub struct PackedIter<'a> {
    len: u32,
    cow: Cow<'a, [u64]>,
    idx: usize,
    pos: usize,
}

pub struct RunIter<'a> {
    boxed: Box<Iterator<Item = u16> + 'a>,
    len: usize,
}

impl Seq16 {
    pub fn iter(&self) -> Iter {
        assert_eq!(self.weight as usize, self.vector.len());
        debug_assert!(self.weight as usize <= super::CAPACITY);
        let iter = (&self.vector[..]).iter();
        Iter::U16(iter.cloned())
    }
}

impl Seq64 {
    pub fn iter(&self) -> Iter {
        debug_assert!(self.weight as usize <= super::CAPACITY);
        let mapped = PackedIter::new(self.weight, Cow::Borrowed(self.vector.as_ref()));
        Iter::U64(mapped)
    }
}

impl Rle16 {
    pub fn iter(&self) -> Iter {
        let len = self.weight as usize;
        let boxed = Box::new(self.ranges.iter().flat_map(|range| {
            (range.start...range.end).into_iter()
        }));
        Iter::Run(RunIter { len, boxed })
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = u16;
    fn next(&mut self) -> Option<Self::Item> {
        match *self {
            Iter::U16(ref mut it) => it.next(),
            Iter::U64(ref mut it) => it.next(),
            Iter::Run(ref mut it) => it.next(),
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        match *self {
            Iter::U16(ref it) => it.size_hint(),
            Iter::U64(ref it) => it.size_hint(),
            Iter::Run(ref it) => it.size_hint(),
        }
    }
}
impl<'a> ExactSizeIterator for Iter<'a> {
    fn len(&self) -> usize {
        match *self {
            Iter::U16(ref it) => it.len(),
            Iter::U64(ref it) => it.len(),
            Iter::Run(ref it) => it.len(),
        }
    }
}

impl<'a> Iterator for PackedIter<'a> {
    type Item = u16;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let i = self.idx;
            let p = self.pos;
            if i >= self.cow.len() {
                return None;
            } else if self.cow[i] & (1u64 << p) != 0 {
                let bit = Some((i * Self::BITS_WIDTH + p) as u16);
                self.move_next();
                self.len -= 1;
                return bit;
            }
            self.move_next();
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len as usize;
        (len, Some(len))
    }
}
impl<'a> ExactSizeIterator for PackedIter<'a> {
    fn len(&self) -> usize {
        self.len as usize
    }
}

impl<'a> Iterator for RunIter<'a> {
    type Item = u16;
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.boxed.next();
        if next.is_some() {
            self.len -= 1;
        }
        next
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}
impl<'a> ExactSizeIterator for RunIter<'a> {
    fn len(&self) -> usize {
        self.len
    }
}

impl<'a> PackedIter<'a> {
    const BITS_WIDTH: usize = 64;

    fn new(len: u32, cow: Cow<'a, [u64]>) -> Self {
        let idx = 0;
        let pos = 0;
        PackedIter { len, cow, idx, pos }
    }
    fn move_next(&mut self) {
        self.pos += 1;
        if self.pos == Self::BITS_WIDTH {
            self.pos = 0;
            self.idx += 1;
        }
    }
}

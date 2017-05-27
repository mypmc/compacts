//! Internal representaion of Block.

#[derive(Debug, Clone)]
pub struct Bucket<T: ::UnsignedInt> {
    pub weight: u32,
    pub vector: Vec<T>,
}

impl<T: ::UnsignedInt> Bucket<T> {
    pub const CAPACITY: usize = 1 << 16;
}

impl Bucket<u16> {
    pub const THRESHOLD: usize = 1 << 12; // 16 * (1 << 12) == 65536
}
impl Bucket<u64> {
    pub const THRESHOLD: usize = 1 << 10; // 64 * (1 << 10) == 65536
}

impl Bucket<u16> {
    pub fn new() -> Self {
        let weight = 0;
        let vector = Vec::new();
        Bucket { weight, vector }
    }

    pub fn with_capacity(cap: usize) -> Self {
        let bounded = if cap <= Self::THRESHOLD {
            cap
        } else {
            Self::THRESHOLD
        };
        let weight = 0;
        let vector = Vec::with_capacity(bounded);
        Bucket { weight, vector }
    }

    pub fn shrink_to_fit(&mut self) {
        self.vector.shrink_to_fit()
    }
}

impl Bucket<u64> {
    pub fn new() -> Self {
        let weight = 0;
        // ensure that length is 1024, this is important for bitops.
        let vector = vec![0; Self::THRESHOLD];
        Bucket { weight, vector }
    }
}

impl Bucket<u16> {
    #[inline]
    fn search(&self, bit: &u16) -> Result<usize, usize> {
        self.vector.binary_search(bit)
    }

    #[inline]
    pub fn contains(&self, bit: u16) -> bool {
        self.search(&bit).is_ok()
    }

    #[inline]
    pub fn insert(&mut self, bit: u16) -> bool {
        let ok = self.search(&bit)
            .map_err(|i| self.vector.insert(i, bit))
            .is_err();
        if ok {
            self.weight += 1;
        }
        ok
    }

    #[inline]
    pub fn remove(&mut self, bit: u16) -> bool {
        let ok = self.search(&bit).map(|i| self.vector.remove(i)).is_ok();
        if ok {
            self.weight -= 1;
        }
        ok
    }
}

impl Bucket<u64> {
    #[inline]
    fn check(&self, key: usize, mask: u64) -> Option<bool> {
        self.vector.get(key).map(|&bit| bit & mask != 0)
    }
    #[inline]
    fn check_or(&self, def: bool, key: usize, mask: u64) -> bool {
        self.check(key, mask).unwrap_or(def)
    }

    #[inline]
    pub fn contains(&self, bit: u16) -> bool {
        bitmask!(bit, key, mask);
        self.check_or(false, key, mask)
    }

    #[inline]
    pub fn insert(&mut self, bit: u16) -> bool {
        bitmask!(bit, key, mask);
        if self.check_or(false, key, mask) {
            false
        } else {
            self.vector[key] |= mask;
            self.weight += 1;
            true
        }
    }

    #[inline]
    pub fn remove(&mut self, bit: u16) -> bool {
        bitmask!(bit, key, mask);
        if self.check_or(false, key, mask) {
            self.vector[key] &= !mask;
            self.weight -= 1;
            true
        } else {
            false
        }
    }
}

/// Get a sorted bucket from a mapped.
impl From<Bucket<u64>> for Bucket<u16> {
    fn from(that: Bucket<u64>) -> Self {
        Bucket::from(&that)
    }
}
impl<'r> From<&'r Bucket<u64>> for Bucket<u16> {
    fn from(that: &Bucket<u64>) -> Self {
        use std::u16;
        let mut bucket = Bucket::with_capacity(that.weight as usize);
        let iter = that.vector.iter();
        for (i, w) in iter.cloned().enumerate().filter(|&(_, v)| v != 0) {
            for p in 0..<u64 as ::UnsignedInt>::WIDTH {
                if w & (1 << p) != 0 {
                    let bit = i * <u64 as ::UnsignedInt>::WIDTH + p;
                    debug_assert!(bit <= u16::MAX as usize);
                    bucket.insert(bit as u16);
                }
            }
        }
        bucket
    }
}

/// Get a mapped bucket from a sorted.
impl From<Bucket<u16>> for Bucket<u64> {
    fn from(that: Bucket<u16>) -> Self {
        Bucket::from(&that)
    }
}
impl<'r> From<&'r Bucket<u16>> for Bucket<u64> {
    fn from(that: &'r Bucket<u16>) -> Self {
        let mut bucket: Self = Bucket::<u64>::new();
        extend_by_u16!(bucket, that.iter());
        bucket
    }
}

impl From<Vec<u16>> for Bucket<u16> {
    fn from(vector: Vec<u16>) -> Self {
        debug_assert!(vector.len() <= Self::CAPACITY);
        let weight = vector.len() as u32;
        Bucket { weight, vector }
    }
}
impl<'a> From<&'a [u16]> for Bucket<u16> {
    fn from(slice: &'a [u16]) -> Self {
        debug_assert!(slice.len() <= Self::CAPACITY);
        let weight = slice.len() as u32;
        let vector = slice.to_owned();
        Bucket { weight, vector }
    }
}

impl FromIterator<u16> for Bucket<u16> {
    fn from_iter<I>(i: I) -> Self
        where I: IntoIterator<Item = u16>
    {
        let iter = i.into_iter();
        Bucket::from(iter.collect::<Vec<u16>>())
    }
}
impl<'a> FromIterator<&'a u16> for Bucket<u16> {
    fn from_iter<I>(i: I) -> Self
        where I: IntoIterator<Item = &'a u16>
    {
        let iter = i.into_iter();
        Bucket::from_iter(iter.cloned())
    }
}

impl From<Vec<u64>> for Bucket<u64> {
    fn from(vector: Vec<u64>) -> Self {
        let weight = {
            let iter = vector.iter().take(Self::THRESHOLD);
            iter.fold(0, |acc, w| acc + w.count_ones())
        };
        Bucket { weight, vector }
    }
}
impl<'a> From<&'a [u64]> for Bucket<u64> {
    fn from(slice: &'a [u64]) -> Self {
        let weight = {
            let iter = slice.iter().take(Self::THRESHOLD);
            iter.fold(0, |acc, w| acc + w.count_ones())
        };
        let vector = slice.to_owned();
        Bucket { weight, vector }
    }
}

impl FromIterator<u16> for Bucket<u64> {
    fn from_iter<I>(i: I) -> Self
        where I: IntoIterator<Item = u16>
    {
        let iter = i.into_iter();
        let mut bucket: Self = Bucket::<u64>::new();
        let ones = extend_by_u16!(bucket, iter);
        debug_assert_eq!(ones, bucket.weight);
        bucket
    }
}
impl<'a> FromIterator<&'a u16> for Bucket<u64> {
    fn from_iter<I>(i: I) -> Self
        where I: IntoIterator<Item = &'a u16>
    {
        let iter = i.into_iter();
        Bucket::from_iter(iter.cloned())
    }
}

use std::iter::{Cloned, Iterator, FromIterator, ExactSizeIterator};
use std::slice::Iter as SliceIter;
use std::vec::IntoIter as VecIntoIter;
use std::borrow::Cow;

type ClonedIter<'a, T> = Cloned<SliceIter<'a, T>>;

pub enum Iter<'a> {
    U16(ClonedIter<'a, u16>),
    U64(PackedIter<'a>),
}
pub enum IntoIter {
    U16(VecIntoIter<u16>),
    U64(PackedIter<'static>),
}

impl Bucket<u16> {
    pub fn iter(&self) -> Iter {
        assert_eq!(self.weight as usize, self.vector.len());
        debug_assert!(self.weight as usize <= Self::CAPACITY);
        let iter = (&self.vector[..]).iter();
        Iter::U16(iter.cloned())
    }
}
impl IntoIterator for Bucket<u16> {
    type Item = u16;
    type IntoIter = IntoIter;
    fn into_iter(self) -> IntoIter {
        assert_eq!(self.weight as usize, self.vector.len());
        debug_assert!(self.weight as usize <= Self::CAPACITY);
        let iter = self.vector.into_iter();
        IntoIter::U16(iter)
    }
}

impl Bucket<u64> {
    pub fn iter(&self) -> Iter {
        debug_assert!(self.weight as usize <= Self::CAPACITY);
        let mapped = PackedIter::new(self.weight, Cow::Borrowed(self.vector.as_ref()));
        Iter::U64(mapped)
    }
}
impl IntoIterator for Bucket<u64> {
    type Item = u16;
    type IntoIter = IntoIter;
    fn into_iter(self) -> IntoIter {
        debug_assert!(self.weight as usize <= Self::CAPACITY);
        let mapped = PackedIter::new(self.weight, Cow::Owned(self.vector));
        IntoIter::U64(mapped)
    }
}

pub struct PackedIter<'a> {
    len: u32,
    cow: Cow<'a, [u64]>,
    idx: usize,
    pos: usize,
}

impl<'a> Iterator for Iter<'a> {
    type Item = u16;
    fn next(&mut self) -> Option<Self::Item> {
        match *self {
            Iter::U16(ref mut it) => it.next(),
            Iter::U64(ref mut it) => it.next(),
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        match *self {
            Iter::U16(ref it) => it.size_hint(),
            Iter::U64(ref it) => it.size_hint(),
        }
    }
}
impl<'a> ExactSizeIterator for Iter<'a> {
    fn len(&self) -> usize {
        match *self {
            Iter::U16(ref it) => it.len(),
            Iter::U64(ref it) => it.len(),
        }
    }
}

impl<'a> Iterator for IntoIter {
    type Item = u16;
    fn next(&mut self) -> Option<Self::Item> {
        match *self {
            IntoIter::U16(ref mut it) => it.next(),
            IntoIter::U64(ref mut it) => it.next(),
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        match *self {
            IntoIter::U16(ref it) => it.size_hint(),
            IntoIter::U64(ref it) => it.size_hint(),
        }
    }
}
impl ExactSizeIterator for IntoIter {
    fn len(&self) -> usize {
        match *self {
            IntoIter::U16(ref it) => it.len(),
            IntoIter::U64(ref it) => it.len(),
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

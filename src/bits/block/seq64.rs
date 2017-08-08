use std::iter::{ExactSizeIterator, FromIterator};
use std::borrow::Cow;
use std::mem;
use super::{Block, Rle16, Seq16, Seq64};

pub struct Seq64Iter<'a> {
    len: u32,
    cow: Cow<'a, [u64]>,
    idx: usize,
    pos: usize,
}

impl Seq64 {
    pub const THRESHOLD: usize = 1 << 10; // 64 * (1 << 10) == 65536

    pub fn new() -> Self {
        let weight = 0;
        // ensure that length is 1024, this is important for bitops.
        let vector = vec![0; Self::THRESHOLD];
        Seq64 { weight, vector }
    }

    pub fn iter(&self) -> Seq64Iter {
        assert!(self.weight as usize <= Block::CAPACITY);
        Seq64Iter::new(self.weight, Cow::Borrowed(self.vector.as_ref()))
    }

    pub fn count_ones(&self) -> u32 {
        self.weight
    }

    pub fn count_zeros(&self) -> u32 {
        Block::CAPACITY as u32 - self.count_ones()
    }

    pub fn size(length_of_u64: usize) -> usize {
        length_of_u64 * mem::size_of::<u64>() + mem::size_of::<u32>()
    }

    pub fn mem_size(&self) -> usize {
        // seq64 has fixed size
        Self::size(1024)
    }

    #[inline]
    pub fn check(&self, key: usize, mask: u64) -> Option<bool> {
        self.vector.get(key).map(|&bit| bit & mask != 0)
    }

    #[inline]
    pub fn check_or(&self, def: bool, key: usize, mask: u64) -> bool {
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

    fn insert_range(&mut self, range: &::std::ops::RangeInclusive<u16>) {
        const WIDTH: usize = 64;
        let s = range.start as usize;
        let e = range.end as usize;
        let sw = s / WIDTH;
        let ew = e / WIDTH;

        let (head, last) = range_of(s, e + 1);

        if sw == ew {
            self.vector[sw] |= head & last;
        } else {
            self.vector[sw] |= head;
            self.vector[ew] |= last;
            for i in (sw + 1)..ew {
                self.vector[i] = !0;
            }
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

fn range_of(idx: usize, end: usize) -> (u64, u64) {
    let x = !0 << (idx % 64);
    let y = !0 >> ((-(end as i64)) as u64 % 64);
    (x, y)
}

impl From<Seq16> for Seq64 {
    fn from(that: Seq16) -> Self {
        Seq64::from(&that)
    }
}
impl<'r> From<&'r Seq16> for Seq64 {
    fn from(that: &'r Seq16) -> Self {
        let mut vec64 = Seq64::new();
        extend_by_u16!(vec64, that.iter());
        vec64
    }
}

impl From<Rle16> for Seq64 {
    fn from(that: Rle16) -> Self {
        Seq64::from(&that)
    }
}
impl<'r> From<&'r Rle16> for Seq64 {
    fn from(that: &'r Rle16) -> Self {
        let mut seq = Seq64::new();
        seq.weight = that.weight;
        for r in &that.ranges {
            seq.insert_range(r);
        }
        seq
    }
}

impl<'a> FromIterator<&'a u16> for Seq64 {
    fn from_iter<I>(i: I) -> Self
    where
        I: IntoIterator<Item = &'a u16>,
    {
        let iter = i.into_iter();
        Seq64::from_iter(iter.cloned())
    }
}
impl FromIterator<u16> for Seq64 {
    fn from_iter<I>(i: I) -> Self
    where
        I: IntoIterator<Item = u16>,
    {
        let iter = i.into_iter();
        let mut vec64 = Seq64::new();
        let ones = extend_by_u16!(vec64, iter);
        debug_assert_eq!(ones, vec64.weight);
        vec64
    }
}

impl<'a> ::bits::IntersectionWith<&'a Seq64> for Seq64 {
    fn intersection_with(&mut self, seq64: &'a Seq64) {
        assert_eq!(self.vector.len(), seq64.vector.len());
        self.weight = {
            let mut new = 0;
            for (x, y) in self.vector.iter_mut().zip(&seq64.vector) {
                *x &= *y;
                new += x.count_ones();
            }
            new
        };
    }
}

impl<'a> ::bits::UnionWith<&'a Seq64> for Seq64 {
    fn union_with(&mut self, seq64: &'a Seq64) {
        assert_eq!(self.vector.len(), seq64.vector.len());
        self.weight = {
            let mut new = 0;
            for (x, y) in self.vector.iter_mut().zip(&seq64.vector) {
                *x |= *y;
                new += x.count_ones();
            }
            new
        };
    }
}

impl<'a> ::bits::DifferenceWith<&'a Seq64> for Seq64 {
    fn difference_with(&mut self, seq64: &'a Seq64) {
        assert_eq!(self.vector.len(), seq64.vector.len());
        self.weight = {
            let mut new = 0;
            for (x, y) in self.vector.iter_mut().zip(&seq64.vector) {
                *x &= !*y;
                new += x.count_ones();
            }
            new
        };
    }
}

impl<'a> ::bits::SymmetricDifferenceWith<&'a Seq64> for Seq64 {
    fn symmetric_difference_with(&mut self, seq64: &'a Seq64) {
        assert_eq!(self.vector.len(), seq64.vector.len());
        self.weight = {
            let mut new = 0;
            for (x, y) in self.vector.iter_mut().zip(&seq64.vector) {
                *x ^= *y;
                new += x.count_ones();
            }
            new
        };
    }
}

impl<'a> Seq64Iter<'a> {
    const BITS_WIDTH: usize = 64;

    fn new(len: u32, cow: Cow<'a, [u64]>) -> Self {
        let idx = 0;
        let pos = 0;
        Seq64Iter { len, cow, idx, pos }
    }

    fn move_next(&mut self) {
        self.pos += 1;
        if self.pos == Self::BITS_WIDTH {
            self.pos = 0;
            self.idx += 1;
        }
    }
}

impl<'a> Iterator for Seq64Iter<'a> {
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

impl<'a> ExactSizeIterator for Seq64Iter<'a> {
    fn len(&self) -> usize {
        self.len as usize
    }
}

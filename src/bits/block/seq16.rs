use std::{iter, mem, slice};
use super::{Block, Rle16, Seq16, Seq64};

pub type Seq16Iter<'a> = iter::Cloned<slice::Iter<'a, u16>>;

impl Seq16 {
    pub const THRESHOLD: usize = 1 << 12; // 16 * (1 << 12) == 65536

    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(cap: usize) -> Self {
        let bounded = if cap <= Self::THRESHOLD {
            cap
        } else {
            Self::THRESHOLD
        };
        let weight = 0;
        let vector = Vec::with_capacity(bounded);
        Seq16 { weight, vector }
    }

    pub fn iter(&self) -> Seq16Iter {
        assert_eq!(self.weight as usize, self.vector.len());
        assert!(self.weight as usize <= Block::CAPACITY);
        let iter = (&self.vector[..]).iter();
        iter.cloned()
    }

    pub fn count_ones(&self) -> u32 {
        self.weight
    }

    pub fn count_zeros(&self) -> u32 {
        Block::CAPACITY as u32 - self.count_ones()
    }

    pub fn size(weight: usize) -> usize {
        weight * mem::size_of::<u16>() + mem::size_of::<u32>()
    }

    pub fn mem_size(&self) -> usize {
        Self::size(self.weight as usize)
    }

    #[inline]
    pub fn search(&self, bit: &u16) -> Result<usize, usize> {
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

impl From<Seq64> for Seq16 {
    fn from(that: Seq64) -> Self {
        Seq16::from(&that)
    }
}
impl<'r> From<&'r Seq64> for Seq16 {
    fn from(that: &Seq64) -> Self {
        use std::u16;
        let mut vec16 = Seq16::with_capacity(that.weight as usize);
        let iter = that.vector.iter();
        for (i, w) in iter.cloned().enumerate().filter(|&(_, v)| v != 0) {
            for p in 0..64 {
                if w & (1 << p) != 0 {
                    let bit = i * 64 + p;
                    debug_assert!(bit <= u16::MAX as usize);
                    vec16.insert(bit as u16);
                }
            }
        }
        vec16
    }
}

impl From<Rle16> for Seq16 {
    fn from(that: Rle16) -> Self {
        Seq16::from(&that)
    }
}
impl<'r> From<&'r Rle16> for Seq16 {
    fn from(that: &'r Rle16) -> Self {
        let mut seq16 = Seq16::with_capacity(that.weight as usize);
        seq16.weight = that.weight;
        for range in &that.ranges {
            seq16.vector.extend(range.clone());
        }
        seq16
    }
}

fn from_sorted_vec(vector: Vec<u16>) -> Seq16 {
    assert!(vector.len() <= Block::CAPACITY);
    let weight = vector.len() as u32;
    Seq16 { weight, vector }
}

impl From<Vec<u16>> for Seq16 {
    fn from(vector: Vec<u16>) -> Self {
        let mut vector = vector;
        vector.sort();
        vector.dedup();
        assert!(vector.len() <= Block::CAPACITY);
        let weight = vector.len() as u32;
        Seq16 { weight, vector }
    }
}

impl<'a> ::bits::IntersectionWith<&'a Seq16> for Seq16 {
    fn intersection_with(&mut self, seq16: &'a Seq16) {
        let data = ::bits::intersection(self.iter(), seq16.iter()).collect::<Vec<u16>>();
        *self = from_sorted_vec(data);
    }
}

impl<'a> ::bits::UnionWith<&'a Seq16> for Seq16 {
    fn union_with(&mut self, seq16: &'a Seq16) {
        let data = ::bits::union(self.iter(), seq16.iter()).collect::<Vec<u16>>();
        *self = from_sorted_vec(data);
    }
}

impl<'a> ::bits::DifferenceWith<&'a Seq16> for Seq16 {
    fn difference_with(&mut self, seq16: &'a Seq16) {
        let data = ::bits::difference(self.iter(), seq16.iter()).collect::<Vec<u16>>();
        *self = from_sorted_vec(data);
    }
}

impl<'a> ::bits::SymmetricDifferenceWith<&'a Seq16> for Seq16 {
    fn symmetric_difference_with(&mut self, seq16: &'a Seq16) {
        let data = ::bits::symmetric_difference(self.iter(), seq16.iter()).collect::<Vec<u16>>();
        *self = from_sorted_vec(data);
    }
}

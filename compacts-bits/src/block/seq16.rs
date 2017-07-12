use std::iter::FromIterator;
use super::{Seq16, Seq64, Rle16};
use Rank;

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
            for p in 0..<u64 as ::UnsignedInt>::WIDTH {
                if w & (1 << p) != 0 {
                    let bit = i * <u64 as ::UnsignedInt>::WIDTH + p;
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

impl From<Vec<u16>> for Seq16 {
    fn from(vector: Vec<u16>) -> Self {
        debug_assert!(vector.len() <= super::CAPACITY);
        let weight = vector.len() as u32;
        Seq16 { weight, vector }
    }
}

impl FromIterator<u16> for Seq16 {
    fn from_iter<I>(i: I) -> Self
    where
        I: IntoIterator<Item = u16>,
    {
        let iter = i.into_iter();
        Seq16::from(iter.collect::<Vec<u16>>())
    }
}
impl<'a> FromIterator<&'a u16> for Seq16 {
    fn from_iter<I>(i: I) -> Self
    where
        I: IntoIterator<Item = &'a u16>,
    {
        let iter = i.into_iter();
        Seq16::from_iter(iter.cloned())
    }
}

impl<'a> ::ops::IntersectionWith<&'a Seq16> for Seq16 {
    fn intersection_with(&mut self, seq16: &'a Seq16) {
        let data = ::pairwise::intersection(self.iter(), seq16.iter()).collect();
        *self = data;
    }
}

impl<'a> ::ops::IntersectionWith<&'a Seq64> for Seq16 {
    fn intersection_with(&mut self, seq64: &'a Seq64) {
        let weight = {
            let mut new = 0;
            for i in 0..self.vector.len() {
                if seq64.contains(self.vector[i]) {
                    self.vector[new] = self.vector[i];
                    new += 1;
                }
            }
            new
        };
        self.vector.truncate(weight);
        self.weight = weight as u32;
    }
}

impl<'a> ::ops::UnionWith<&'a Seq16> for Seq16 {
    fn union_with(&mut self, seq16: &'a Seq16) {
        let data = ::pairwise::union(self.iter(), seq16.iter()).collect();
        *self = data;
    }
}

impl<'a> ::ops::DifferenceWith<&'a Seq16> for Seq16 {
    fn difference_with(&mut self, seq16: &'a Seq16) {
        let data = ::pairwise::difference(self.iter(), seq16.iter()).collect();
        *self = data;
    }
}

impl<'a> ::ops::SymmetricDifferenceWith<&'a Seq16> for Seq16 {
    fn symmetric_difference_with(&mut self, seq16: &'a Seq16) {
        let data = ::pairwise::symmetric_difference(self.iter(), seq16.iter()).collect();
        *self = data;
    }
}

impl ::Rank<u16> for Seq16 {
    type Weight = u32;

    const SIZE: Self::Weight = super::CAPACITY as u32;

    fn rank1(&self, i: u16) -> Self::Weight {
        if i as usize >= super::CAPACITY {
            return self.count_ones();
        }
        let vec = &self.vector;
        let fun = |j| vec.get(j).map_or(false, |&v| v >= i);
        let k = search!(0, vec.len(), fun);
        (if k < vec.len() && vec[k] == i {
             k + 1 // found
         } else {
             k // not found
         }) as Self::Weight
    }

    fn rank0(&self, i: u16) -> Self::Weight {
        i as Self::Weight + 1 - self.rank1(i)
    }
}

impl ::Select1<u16> for Seq16 {
    fn select1(&self, c: u16) -> Option<u16> {
        if c as u32 >= self.count_ones() {
            return None;
        }
        self.vector.get(c as usize).cloned()
    }
}

impl ::Select0<u16> for Seq16 {
    fn select0(&self, c: u16) -> Option<u16> {
        let c32 = c as u32;
        if c32 >= self.count_zeros() {
            return None;
        }
        let cap = super::CAPACITY as u32;
        let fun = |i| {
            let i = i as u16;
            let rank = self.rank0(i);
            rank > c as u32
        };
        let pos = search!(0, cap, fun);
        if pos < super::CAPACITY as u32 {
            Some(pos as u16)
        } else {
            None
        }
    }
}

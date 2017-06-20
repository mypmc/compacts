use std::iter::FromIterator;
use super::{Seq16, Seq64, Rle16};

impl Seq64 {
    pub const THRESHOLD: usize = 1 << 10; // 64 * (1 << 10) == 65536

    pub fn new() -> Self {
        let weight = 0;
        // ensure that length is 1024, this is important for bitops.
        let vector = vec![0; Self::THRESHOLD];
        Seq64 { weight, vector }
    }

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

    fn insert_range(&mut self, range: &::std::ops::RangeInclusive<u16>) {
        const WIDTH: usize = <u64 as ::UnsignedInt>::WIDTH;
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

impl<'a> ::ops::IntersectionWith<&'a Seq16> for Seq64 {
    fn intersection_with(&mut self, seq16: &'a Seq16) {
        let seq = Self::from(seq16);
        self.intersection_with(&seq);
    }
}

impl<'a> ::ops::IntersectionWith<&'a Seq64> for Seq64 {
    fn intersection_with(&mut self, seq64: &'a Seq64) {
        assert_eq!(self.vector.len(), seq64.vector.len());
        self.weight = {
            let mut new = 0;
            for (x, y) in self.vector.iter_mut().zip(&seq64.vector) {
                x.intersection_with(*y);
                new += x.count_ones();
            }
            new
        };
    }
}

impl<'a> ::ops::IntersectionWith<&'a Rle16> for Seq64 {
    fn intersection_with(&mut self, rle16: &'a Rle16) {
        let seq = Self::from(rle16);
        self.intersection_with(&seq);
    }
}


impl<'a> ::ops::UnionWith<&'a Seq16> for Seq64 {
    fn union_with(&mut self, seq16: &'a Seq16) {
        for &bit in &seq16.vector {
            self.insert(bit);
        }
    }
}

impl<'a> ::ops::UnionWith<&'a Seq64> for Seq64 {
    fn union_with(&mut self, seq64: &'a Seq64) {
        assert_eq!(self.vector.len(), seq64.vector.len());
        self.weight = {
            let mut new = 0;
            for (x, y) in self.vector.iter_mut().zip(&seq64.vector) {
                x.union_with(*y);
                new += x.count_ones();
            }
            new
        };
    }
}

impl<'a> ::ops::UnionWith<&'a Rle16> for Seq64 {
    fn union_with(&mut self, rle16: &'a Rle16) {
        for range in &rle16.ranges {
            for bit in range.start...range.end {
                self.insert(bit);
            }
        }
    }
}

impl<'a> ::ops::DifferenceWith<&'a Seq16> for Seq64 {
    fn difference_with(&mut self, seq16: &'a Seq16) {
        for &bit in &seq16.vector {
            self.remove(bit);
        }
    }
}

impl<'a> ::ops::DifferenceWith<&'a Seq64> for Seq64 {
    fn difference_with(&mut self, seq64: &'a Seq64) {
        assert_eq!(self.vector.len(), seq64.vector.len());
        self.weight = {
            let mut new = 0;
            for (x, y) in self.vector.iter_mut().zip(&seq64.vector) {
                x.difference_with(*y);
                new += x.count_ones();
            }
            new
        };
    }
}

impl<'a> ::ops::DifferenceWith<&'a Rle16> for Seq64 {
    fn difference_with(&mut self, rle16: &'a Rle16) {
        for range in &rle16.ranges {
            for bit in range.start...range.end {
                self.remove(bit);
            }
        }
    }
}

impl<'a> ::ops::SymmetricDifferenceWith<&'a Seq16> for Seq64 {
    fn symmetric_difference_with(&mut self, seq16: &'a Seq16) {
        for &bit in &seq16.vector {
            if self.contains(bit) {
                self.remove(bit);
            } else {
                self.insert(bit);
            }
        }
    }
}

impl<'a> ::ops::SymmetricDifferenceWith<&'a Seq64> for Seq64 {
    fn symmetric_difference_with(&mut self, seq64: &'a Seq64) {
        assert_eq!(self.vector.len(), seq64.vector.len());
        self.weight = {
            let mut new = 0;
            for (x, y) in self.vector.iter_mut().zip(&seq64.vector) {
                x.symmetric_difference_with(*y);
                new += x.count_ones();
            }
            new
        };
    }
}

impl<'a> ::ops::SymmetricDifferenceWith<&'a Rle16> for Seq64 {
    fn symmetric_difference_with(&mut self, rle16: &'a Rle16) {
        for range in &rle16.ranges {
            for bit in range.start...range.end {
                if self.contains(bit) {
                    self.remove(bit);
                } else {
                    self.insert(bit);
                }
            }
        }
    }
}

impl ::Rank<u16> for Seq64 {
    type Weight = u32;

    fn size(&self) -> Self::Weight {
        super::CAPACITY as u32
    }

    fn rank1(&self, i: u16) -> Self::Weight {
        if i as usize >= super::CAPACITY {
            return self.count_ones();
        }
        let q = i as usize / <u64 as ::UnsignedInt>::WIDTH;
        let r = i as u32 % <u64 as ::UnsignedInt>::WIDTH as u32;
        let vec = &self.vector;
        vec.iter().take(q).fold(0, |acc, w| acc + w.count_ones()) +
            vec.get(q).map_or(0, |w| w.rank1(r))
    }
}

impl ::Select1<u16> for Seq64 {
    fn select1(&self, c: u16) -> Option<u16> {
        let c32 = c as u32;
        if c32 >= self.count_ones() {
            return None;
        }
        let mut rem = c32;
        for (i, bit) in self.vector.iter().enumerate() {
            let ones = bit.count_ones();
            if rem < ones {
                let select = bit.select1(rem).unwrap_or(0);
                return Some((<u64 as ::UnsignedInt>::WIDTH * i) as u16 + select as u16);
            }
            rem -= ones;
        }
        None
    }
}

impl ::Select0<u16> for Seq64 {
    fn select0(&self, c: u16) -> Option<u16> {
        let c32 = c as u32;
        if c32 >= self.count_zeros() {
            return None;
        }
        let mut rem = c32;
        for (i, bit) in self.vector.iter().enumerate() {
            let zeros = bit.count_zeros();
            if rem < zeros {
                let select = bit.select0(rem).unwrap_or(0);
                return Some((<u64 as ::UnsignedInt>::WIDTH * i) as u16 + select as u16);
            }
            rem -= zeros;
        }
        None
    }
}

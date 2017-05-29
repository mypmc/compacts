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
    let x = !0 << idx % 64;
    let y = !0 >> ((-(end as i64)) as u64 % 64);
    return (x, y);
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
            // unimplemented!()
        }
        seq
    }
}

impl<'a> FromIterator<&'a u16> for Seq64 {
    fn from_iter<I>(i: I) -> Self
        where I: IntoIterator<Item = &'a u16>
    {
        let iter = i.into_iter();
        Seq64::from_iter(iter.cloned())
    }
}
impl FromIterator<u16> for Seq64 {
    fn from_iter<I>(i: I) -> Self
        where I: IntoIterator<Item = u16>
    {
        let iter = i.into_iter();
        let mut vec64 = Seq64::new();
        let ones = extend_by_u16!(vec64, iter);
        debug_assert_eq!(ones, vec64.weight);
        vec64
    }
}

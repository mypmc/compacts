use std::ops;
use std::iter::FromIterator;
use super::{Run16, Seq16};

#[derive(Clone)]
pub(crate) struct Arr64 {
    pub weight: u32,
    // pub vector: Vec<u64>,
    pub vector: Box<[u64; 1024]>,
}

impl PartialEq for Arr64 {
    fn eq(&self, that: &Arr64) -> bool {
        let length_test = self.vector.len() == that.vector.len();
        let weight_test = self.weight == that.weight;
        let vector_test = self.vector
            .iter()
            .zip(that.vector.iter())
            .all(|(v1, v2)| v1 == v2);
        length_test && weight_test && vector_test
    }
}
impl Eq for Arr64 {}

impl Default for Arr64 {
    fn default() -> Self {
        let weight = 0;
        // let vector = vec![0; 1024];
        let vector = Box::new([0; 1024]);
        Arr64 { weight, vector }
    }
}

impl Arr64 {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    fn check(&self, key: usize, mask: u64) -> Option<bool> {
        self.vector.get(key).map(|&bit| bit & mask != 0)
    }

    #[inline]
    pub fn contains(&self, bit: u16) -> bool {
        bitmask!(bit, key, mask);
        self.check(key, mask).unwrap_or_default()
    }

    #[inline]
    pub fn insert(&mut self, bit: u16) -> bool {
        bitmask!(bit, key, mask);
        if self.check(key, mask).unwrap_or_default() {
            false
        } else {
            self.vector[key] |= mask;
            self.weight += 1;
            true
        }
    }

    fn insert_range(&mut self, range: &ops::RangeInclusive<u16>) {
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
        if self.check(key, mask).unwrap_or_default() {
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

impl From<Seq16> for Arr64 {
    fn from(that: Seq16) -> Self {
        Arr64::from(&that)
    }
}
impl<'r> From<&'r Seq16> for Arr64 {
    fn from(that: &'r Seq16) -> Self {
        let mut vec64 = Arr64::new();
        extend_by_u16!(vec64, that);
        vec64
    }
}

impl From<Run16> for Arr64 {
    fn from(that: Run16) -> Self {
        Arr64::from(&that)
    }
}
impl<'r> From<&'r Run16> for Arr64 {
    fn from(that: &'r Run16) -> Self {
        let mut seq = Arr64::new();
        seq.weight = that.weight;
        for r in &that.ranges {
            seq.insert_range(r);
        }
        seq
    }
}

impl<'a> FromIterator<&'a u16> for Arr64 {
    fn from_iter<I>(i: I) -> Self
    where
        I: IntoIterator<Item = &'a u16>,
    {
        let iter = i.into_iter();
        Arr64::from_iter(iter.cloned())
    }
}
impl FromIterator<u16> for Arr64 {
    fn from_iter<I>(i: I) -> Self
    where
        I: IntoIterator<Item = u16>,
    {
        let iter = i.into_iter();
        let mut vec64 = Arr64::new();
        let ones = extend_by_u16!(vec64, iter);
        debug_assert_eq!(ones, vec64.weight);
        vec64
    }
}

impl<'a> ::bits::Assign<&'a Arr64> for Arr64 {
    fn and_assign(&mut self, arr64: &'a Arr64) {
        assert_eq!(self.vector.len(), arr64.vector.len());
        self.weight = {
            let mut new = 0;
            for (x, y) in self.vector.iter_mut().zip(arr64.vector.iter()) {
                *x &= *y;
                new += x.count_ones();
            }
            new
        };
    }

    fn or_assign(&mut self, arr64: &'a Arr64) {
        assert_eq!(self.vector.len(), arr64.vector.len());
        self.weight = {
            let mut new = 0;
            for (x, y) in self.vector.iter_mut().zip(arr64.vector.iter()) {
                *x |= *y;
                new += x.count_ones();
            }
            new
        };
    }

    fn and_not_assign(&mut self, arr64: &'a Arr64) {
        assert_eq!(self.vector.len(), arr64.vector.len());
        self.weight = {
            let mut new = 0;
            for (x, y) in self.vector.iter_mut().zip(arr64.vector.iter()) {
                *x &= !*y;
                new += x.count_ones();
            }
            new
        };
    }

    fn xor_assign(&mut self, arr64: &'a Arr64) {
        assert_eq!(self.vector.len(), arr64.vector.len());
        self.weight = {
            let mut new = 0;
            for (x, y) in self.vector.iter_mut().zip(arr64.vector.iter()) {
                *x ^= *y;
                new += x.count_ones();
            }
            new
        };
    }
}

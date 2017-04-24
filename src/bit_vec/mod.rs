use std::collections::BTreeMap;

use {Bounded, PopCount, Bucket};
// use {Rank0, Rank1, Select1, Select0};
use bits::{self, SplitMerge};

// mod iter;
// use self::iter::Iter;

pub struct BitVec {
    pop_count: bits::Count<u32>,
    buckets: BTreeMap<u16, Bucket>,
}

impl PopCount for BitVec {
    const CAPACITY: u64 = Bucket::CAPACITY * Bucket::CAPACITY;

    fn ones(&self) -> u64 {
        self.pop_count.value()
    }
}

impl BitVec {
    pub fn new() -> Self {
        BitVec {
            pop_count: bits::Count::MIN,
            buckets: BTreeMap::new(),
        }
    }

    /// Returns `true` if the specified bit set in BitVec.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cwt::{PopCount, BitVec};
    ///
    /// let mut bits = BitVec::new();
    /// bits.insert(1);
    /// assert_eq!(bits.contains(0), false);
    /// assert_eq!(bits.contains(1), true);
    /// assert_eq!(bits.contains(2), false);
    /// assert_eq!(bits.ones(), 1);
    /// ```
    pub fn contains(&self, x: u32) -> bool {
        let (key, bit) = x.split();
        if let Some(bucket) = self.buckets.get(&key) {
            bucket.contains(bit)
        } else {
            false
        }
    }

    /// Returns `true` if the value doesn't exists in the BitVec, and inserted to the BitVec.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cwt::{PopCount, BitVec};
    ///
    /// let mut bits = BitVec::new();
    /// assert_eq!(bits.insert(3), true);
    /// assert_eq!(bits.insert(3), false);
    /// assert_eq!(bits.contains(3), true);
    /// assert_eq!(bits.ones(), 1);
    /// ```
    pub fn insert(&mut self, x: u32) -> bool {
        let (key, bit) = x.split();
        let mut bucket = self.buckets
            .entry(key)
            .or_insert(Bucket::with_capacity(1));
        let ok = bucket.insert(bit);
        if ok {
            self.pop_count.incr();
        }
        ok
    }

    /// Returns `true` if the value present and removed from the BitVec.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cwt::{PopCount, BitVec};
    ///
    /// let mut bits = BitVec::new();
    /// assert_eq!(bits.insert(3), true);
    /// assert_eq!(bits.remove(3), true);
    /// assert_eq!(bits.contains(3), false);
    /// assert_eq!(bits.ones(), 0);
    /// ```
    pub fn remove(&mut self, x: u32) -> bool {
        let (key, bit) = x.split();
        if let Some(bucket) = self.buckets.get_mut(&key) {
            let ok = bucket.remove(bit);
            if ok {
                self.pop_count.decr();
            }
            return ok;
        }
        return false;
    }
}

//impl BitVec {
//    fn iter<'a>(&'a self) -> Iter<'a> {
//        Iter::new(&self.map)
//    }
//}

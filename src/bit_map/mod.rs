use std::collections::BTreeMap;

use {Bounded, PopCount, Bucket};
// use {Rank0, Rank1, Select1, Select0};
use bits::{self, SplitMerge};

// mod iter;
// use self::iter::Iter;

pub struct BitMap {
    pop: bits::Count<u32>,
    map: BTreeMap<u16, Bucket>,
}

impl PopCount for BitMap {
    const CAPACITY: u64 = Bucket::CAPACITY * Bucket::CAPACITY;

    fn ones(&self) -> u64 {
        self.pop.value()
    }
}

impl BitMap {
    pub fn new() -> Self {
        BitMap {
            pop: bits::Count::MIN,
            map: BTreeMap::new(),
        }
    }

    /// Returns `true` if the specified bit set in BitMap.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cwt::{PopCount, BitMap};
    ///
    /// let mut bits = BitMap::new();
    /// bits.insert(1);
    /// assert_eq!(bits.contains(0), false);
    /// assert_eq!(bits.contains(1), true);
    /// assert_eq!(bits.contains(2), false);
    /// ```
    pub fn contains(&self, x: u32) -> bool {
        let (key, bit) = x.split();
        if let Some(bucket) = self.map.get(&key) {
            bucket.contains(bit)
        } else {
            false
        }
    }

    /// Returns `true` if the value doesn't exists in the BitMap, and inserted to the BitMap.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cwt::{PopCount, BitMap};
    ///
    /// let mut bits = BitMap::new();
    /// assert_eq!(bits.insert(3), true);
    /// assert_eq!(bits.insert(3), false);
    /// assert_eq!(bits.contains(3), true);
    /// assert_eq!(bits.ones(), 1);
    /// ```
    pub fn insert(&mut self, x: u32) -> bool {
        let (key, bit) = x.split();
        let mut bucket = self.map.entry(key).or_insert(Bucket::with_capacity(1));
        let ok = bucket.insert(bit);
        if ok {
            self.pop.incr();
        }
        ok
    }

    /// Returns `true` if the value present and removed from the BitMap.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cwt::{PopCount, BitMap};
    ///
    /// let mut bits = BitMap::new();
    /// assert_eq!(bits.insert(3), true);
    /// assert_eq!(bits.remove(3), true);
    /// assert_eq!(bits.contains(3), false);
    /// assert_eq!(bits.ones(), 0);
    /// ```
    pub fn remove(&mut self, x: u32) -> bool {
        let (key, bit) = x.split();
        if let Some(bucket) = self.map.get_mut(&key) {
            let ok = bucket.remove(bit);
            if ok {
                self.pop.decr();
            }
            return ok;
        }
        return false;
    }
}

//impl BitMap {
//    fn iter<'a>(&'a self) -> Iter<'a> {
//        Iter::new(&self.map)
//    }
//}

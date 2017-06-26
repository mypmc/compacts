mod pairwise;

use std::collections::BTreeMap;
use {Vec32, Split, Merge};

/// Map of Vec32.
#[derive(Clone, Debug)]
pub struct Vec64 {
    vec32s: BTreeMap<u32, Vec32>,
}

impl Default for Vec64 {
    fn default() -> Self {
        let vec32s = BTreeMap::new();
        Vec64 { vec32s }
    }
}

impl Vec64 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.vec32s.clear()
    }

    pub fn count_ones(&self) -> u128 {
        let mut r = 0;
        for w in self.vec32s.iter().map(|(_, vec)| vec.count_ones() as u128) {
            r += w;
        }
        r
    }

    pub fn count_zeros(&self) -> u128 {
        (1 << 64) - self.count_ones()
    }

    pub fn mem_size(&self) -> u128 {
        let mut sum = 0;
        for mem in self.vec32s.values().map(|vec| vec.mem_size() as u128) {
            sum += mem;
        }
        sum
    }

    pub fn optimize(&mut self) {
        let mut rs = Vec::new();
        for (k, vec) in self.vec32s.iter_mut() {
            vec.optimize();
            if vec.count_ones() == 0 {
                rs.push(*k);
            }
        }
        for k in rs {
            self.vec32s.remove(&k);
        }
    }

    /// Return `true` if the value exists.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compacts_bits::Vec64;
    /// let mut bits = Vec64::new();
    /// assert!(!bits.contains(1 << 50));
    /// bits.insert(1 << 50);
    /// assert!(bits.contains(1 << 50));
    /// assert_eq!(1, bits.count_ones());
    /// ```
    pub fn contains(&self, x: u64) -> bool {
        let (key, bit) = x.split();
        self.vec32s.get(&key).map_or(false, |b| b.contains(bit))
    }

    /// Return `true` if the value doesn't exists and inserted successfuly.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compacts_bits::Vec64;
    /// let mut bits = Vec64::new();
    /// assert!(bits.insert(1 << 50));
    /// assert!(!bits.insert(1 << 50));
    /// assert_eq!(1, bits.count_ones());
    /// ```
    pub fn insert(&mut self, x: u64) -> bool {
        let (key, bit) = x.split();
        let mut bv = self.vec32s.entry(key).or_insert_with(Vec32::new);
        bv.insert(bit)
    }

    /// Return `true` if the value exists and removed successfuly.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compacts_bits::Vec64;
    /// let mut bits = Vec64::new();
    /// assert!(bits.insert(1 << 60));
    /// assert!(bits.remove(1 << 60));
    /// assert_eq!(0, bits.count_ones());
    /// ```
    pub fn remove(&mut self, x: u64) -> bool {
        let (key, bit) = x.split();
        self.vec32s.get_mut(&key).map_or(false, |b| b.remove(bit))
    }

    pub fn iter<'r>(&'r self) -> impl Iterator<Item = u64> + 'r {
        self.vec32s.iter().flat_map(|(&key, vec)| {
            vec.iter().map(move |val| <u64 as Merge>::merge((key, val)))
        })
    }
}

impl ::std::ops::Index<u64> for Vec64 {
    type Output = bool;
    fn index(&self, i: u64) -> &Self::Output {
        if self.contains(i) {
            super::TRUE
        } else {
            super::FALSE
        }
    }
}

impl<'a> ::std::iter::FromIterator<&'a u64> for Vec64 {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a u64>,
    {
        let mut vec = Vec64::new();
        for b in iter {
            vec.insert(*b);
        }
        vec
    }
}

impl<T: AsRef<[u64]>> From<T> for Vec64 {
    fn from(v: T) -> Self {
        v.as_ref().iter().collect()
    }
}

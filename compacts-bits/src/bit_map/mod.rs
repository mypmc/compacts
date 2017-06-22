use std::collections::BTreeMap;
use {BitVec, Split, Merge};

/// Map of BitVec.
#[derive(Clone, Debug)]
pub struct BitMap {
    bits: BTreeMap<u32, BitVec>,
}

impl Default for BitMap {
    fn default() -> Self {
        let bits = BTreeMap::new();
        BitMap { bits }
    }
}

impl BitMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.bits.clear()
    }

    pub fn count_ones(&self) -> u128 {
        let mut r = 0;
        for w in self.bits.iter().map(|(_, vec)| vec.count_ones() as u128) {
            r += w;
        }
        r
    }

    pub fn count_zeros(&self) -> u128 {
        (1 << 64) - self.count_ones()
    }

    pub fn optimize(&mut self) {
        for vec in self.bits.values_mut() {
            vec.optimize();
        }
    }

    pub fn mem_size(&self) -> u128 {
        let mut sum = 0;
        for mem in self.bits.values().map(|vec| vec.mem_size() as u128) {
            sum += mem;
        }
        sum
    }

    /// Return `true` if the value exists.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compacts_bits::BitMap;
    /// let mut bits = BitMap::new();
    /// assert!(!bits.contains(1 << 50));
    /// bits.insert(1 << 50);
    /// assert!(bits.contains(1 << 50));
    /// assert_eq!(1, bits.count_ones());
    /// ```
    pub fn contains(&self, x: u64) -> bool {
        let (key, bit) = x.split();
        self.bits.get(&key).map_or(false, |b| b.contains(bit))
    }

    /// Return `true` if the value doesn't exists and inserted successfuly.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compacts_bits::BitMap;
    /// let mut bits = BitMap::new();
    /// assert!(bits.insert(1 << 50));
    /// assert!(!bits.insert(1 << 50));
    /// assert_eq!(1, bits.count_ones());
    /// ```
    pub fn insert(&mut self, x: u64) -> bool {
        let (key, bit) = x.split();
        let mut bv = self.bits.entry(key).or_insert_with(|| BitVec::new());
        bv.insert(bit)
    }

    /// Return `true` if the value exists and removed successfuly.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compacts_bits::BitMap;
    /// let mut bits = BitMap::new();
    /// assert!(bits.insert(1 << 60));
    /// assert!(bits.remove(1 << 60));
    /// assert_eq!(0, bits.count_ones());
    /// ```
    pub fn remove(&mut self, x: u64) -> bool {
        let (key, bit) = x.split();
        self.bits.get_mut(&key).map_or(false, |b| b.remove(bit))
    }

    pub fn iter<'r>(&'r self) -> impl Iterator<Item = u64> + 'r {
        self.bits.iter().flat_map(|(&key, vec)| {
            vec.iter().map(move |val| <u64 as Merge>::merge((key, val)))
        })
    }
}

impl ::std::ops::Index<u64> for BitMap {
    type Output = bool;
    fn index(&self, i: u64) -> &Self::Output {
        if self.contains(i) {
            super::TRUE
        } else {
            super::FALSE
        }
    }
}

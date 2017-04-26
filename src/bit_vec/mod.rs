#[macro_use]
mod macros;
mod block;
mod pairwise;

use std::collections::BTreeMap;
use std::ops::Index;

use dict::Ranked;
use prim::{self, Split};
use thunk::Thunk;

use self::block::Block;

#[derive(Debug)]
pub struct BitVec<'a> {
    weight: u64,
    blocks: BTreeMap<u16, Thunk<'a, Block>>,
}

impl<'a> Clone for BitVec<'a> {
    fn clone(&self) -> Self {
        let mut vec = BitVec::new();
        for (&k, t) in self.blocks.iter() {
            let c = (**t).clone();
            vec.blocks.insert(k, eval!(c));
        }
        vec
    }
}

impl<'a> BitVec<'a> {
    // const CAPACITY: u64 = 1 << 32;

    pub fn count1(&self) -> u64 {
        self.blocks
            .values()
            .fold(0, |acc, b| acc + b.count1() as u64)
    }
}

impl<'a> BitVec<'a> {
    pub fn count_blocks(&self) -> usize {
        self.blocks.len()
    }

    pub fn optimize(&mut self) {
        for b in self.blocks.values_mut() {
            b.optimize();
        }
    }
}

impl<'a> BitVec<'a> {
    pub fn new() -> Self {
        BitVec {
            weight: 0,
            blocks: BTreeMap::new(),
        }
    }

    /// Clear contents.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cds::BitVec;
    /// let mut bits = BitVec::new();
    /// bits.insert(0);
    /// assert!(bits.count1() == 1);
    /// bits.clear();
    /// assert!(bits.count1() == 0);
    /// ```
    pub fn clear(&mut self) {
        self.weight = 0;
        self.blocks.clear();
    }

    /// Return `true` if the specified bit set in BitVec.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cds::BitVec;
    ///
    /// let mut bits = BitVec::new();
    /// bits.insert(1);
    /// assert!(!bits.contains(0));
    /// assert!(bits.contains(1));
    /// assert!(!bits.contains(2));
    /// assert_eq!(bits.count1(), 1);
    /// ```
    pub fn contains(&self, x: u32) -> bool {
        let (key, bit) = x.split();
        if let Some(b) = self.blocks.get(&key) {
            b.contains(bit)
        } else {
            false
        }
    }

    /// Return `true` if the value doesn't exists in the BitVec,
    /// and inserted to the BitVec successfully.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cds::BitVec;
    ///
    /// let mut bits = BitVec::new();
    /// assert!(bits.insert(3));
    /// assert!(!bits.insert(3));
    /// assert!(bits.contains(3));
    /// assert_eq!(bits.count1(), 1);
    /// ```
    pub fn insert(&mut self, x: u32) -> bool {
        let (key, bit) = x.split();
        let mut b = self.blocks
            .entry(key)
            .or_insert(eval!(Block::with_capacity(64)));
        let ok = b.insert(bit);
        if ok {
            self.weight += 1;
            b.optimize();
        }
        ok
    }

    /// Return `true` if the value present and removed from the BitVec.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cds::BitVec;
    ///
    /// let mut bits = BitVec::new();
    /// assert!(bits.insert(3));
    /// assert!(bits.remove(3));
    /// assert!(!bits.contains(3));
    /// assert_eq!(bits.count1(), 0);
    /// ```
    pub fn remove(&mut self, x: u32) -> bool {
        let (key, bit) = x.split();
        if let Some(b) = self.blocks.get_mut(&key) {
            let ok = b.remove(bit);
            if ok {
                self.weight -= 1;
                b.optimize();
            }
            return ok;
        }
        return false;
    }
}

impl<'a> Index<u32> for BitVec<'a> {
    type Output = bool;
    fn index(&self, i: u32) -> &Self::Output {
        if self.contains(i) {
            prim::TRUE
        } else {
            prim::FALSE
        }
    }
}

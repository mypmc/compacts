#[macro_use]
mod macros;
mod block;
mod pairwise;

use std::collections::BTreeMap;
use std::fmt::{self, Debug, Formatter};
use std::ops::Index;

pub use self::pairwise::{Pairwise, PairwiseWith};
use self::block::Block;

use dict::Ranked;
use dict::prim::{self, Split};
use karabiner::thunk::Thunk;

#[derive(Default)]
pub struct BitVec<'a> {
    blocks: BTreeMap<u16, Thunk<'a, Block>>,
}

impl<'a> Debug for BitVec<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let w = self.count1();
        write!(f, "BitVec(weight={:?})", w)
    }
}
impl<'a> Clone for BitVec<'a> {
    fn clone(&self) -> Self {
        let mut vec = BitVec::new();
        for (&k, t) in &self.blocks {
            let c = (**t).clone();
            vec.blocks.insert(k, eval!(c));
        }
        vec
    }
}

impl<'a> BitVec<'a> {
    pub fn count_blocks(&self) -> usize {
        self.blocks.len()
    }

    pub fn optimize(&mut self) {
        for b in self.blocks.values_mut() {
            Thunk::force(b);
            b.optimize();
        }
    }
}

impl<'a> BitVec<'a> {
    pub fn new() -> Self {
        BitVec { blocks: BTreeMap::new() }
    }

    /// Clear contents.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compacts::bits::BitVec;
    /// use compacts::dict::Ranked;
    ///
    /// let mut bits = BitVec::new();
    /// bits.insert(0);
    /// assert!(bits.count1() == 1);
    /// bits.clear();
    /// assert!(bits.count1() == 0);
    /// ```
    pub fn clear(&mut self) {
        self.blocks.clear();
    }

    /// Return `true` if the specified bit set in BitVec.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compacts::bits::BitVec;
    /// use compacts::dict::Ranked;
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
    /// use compacts::bits::BitVec;
    /// use compacts::dict::Ranked;
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
            .or_insert_with(|| eval!(Block::with_capacity(64)));
        let ok = b.insert(bit);
        if ok {
            b.optimize();
        }
        ok
    }

    /// Return `true` if the value present and removed from the BitVec.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compacts::bits::BitVec;
    /// use compacts::dict::Ranked;
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
                b.optimize();
            }
            return ok;
        }
        false
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

impl<'a> ::dict::Ranked<u32> for BitVec<'a> {
    type Weight = u64;

    fn size(&self) -> Self::Weight {
        const CAPACITY: u64 = 1 << 32;
        CAPACITY
    }

    fn count1(&self) -> u64 {
        self.blocks
            .values()
            .map(|b| Self::Weight::from(b.count1()))
            .sum()
    }

    fn rank1(&self, i: u32) -> Self::Weight {
        let (hi, lo) = i.split();
        let mut rank = 0;
        for (&key, block) in &self.blocks {
            if key > hi {
                break;
            } else if key == hi {
                rank += Self::Weight::from(block.rank1(lo));
                break;
            } else {
                rank += Self::Weight::from(block.count1());
            }
        }
        rank
    }
}

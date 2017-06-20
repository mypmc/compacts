mod stats;
mod pairwise;

use std::fmt::{self, Debug, Formatter};
use std::collections::BTreeMap;
use split_merge::{Split, Merge};
use block::Block;
use karabiner::thunk::Thunk;

pub use self::stats::{Stats, BlockStats};

#[derive(Default)]
pub struct BitVec<'a> {
    blocks: BTreeMap<u16, Thunk<'a, Block>>,
}

impl<'a> Debug for BitVec<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let b = self.count_blocks();
        let w = self.count_ones();
        write!(f, "BitVec{{ blocks={:?} weight={:?} }}", b, w)
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
    pub fn count_ones(&self) -> u64 {
        self.blocks
            .values()
            .map(|b| u64::from(b.count_ones()))
            .sum()
    }

    pub fn count_zeros(&self) -> u64 {
        self.blocks
            .values()
            .map(|b| u64::from(b.count_zeros()))
            .sum()
    }

    fn count_blocks(&self) -> usize {
        self.blocks.len()
    }

    /// Optimize innternal data representaions.
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
    /// use compacts_bits::BitVec;
    ///
    /// let mut bits = BitVec::new();
    /// bits.set(0);
    /// assert!(bits.count_ones() == 1);
    /// bits.clear();
    /// assert!(bits.count_ones() == 0);
    /// ```
    pub fn clear(&mut self) {
        self.blocks.clear();
    }

    /// Return `true` if the specified bit set in BitVec.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compacts_bits::BitVec;
    ///
    /// let mut bits = BitVec::new();
    /// bits.set(1);
    /// assert!(!bits.get(0));
    /// assert!(bits.get(1));
    /// assert!(!bits.get(2));
    /// assert_eq!(bits.count_ones(), 1);
    /// ```
    pub fn get(&self, x: u32) -> bool {
        let (key, bit) = x.split();
        if let Some(b) = self.blocks.get(&key) {
            b.contains(bit)
        } else {
            false
        }
    }

    /// Return `true` if the value doesn't exists in the BitVec.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compacts_bits::BitVec;
    ///
    /// let mut bits = BitVec::new();
    /// assert!(bits.set(3));
    /// assert!(!bits.set(3));
    /// assert!(bits.get(3));
    /// assert_eq!(bits.count_ones(), 1);
    /// ```
    pub fn set(&mut self, x: u32) -> bool {
        if self.get(x) {
            false
        } else {
            let (key, bit) = x.split();
            let mut b = self.blocks.entry(key).or_insert_with(
                || eval!(Block::new()),
            );
            b.insert(bit)
        }
    }

    /// Return `true` if the value present and removed from the BitVec.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compacts_bits::BitVec;
    ///
    /// let mut bits = BitVec::new();
    /// assert!(bits.set(3));
    /// assert!(bits.remove(3));
    /// assert!(!bits.get(3));
    /// assert_eq!(bits.count_ones(), 0);
    /// ```
    pub fn remove(&mut self, x: u32) -> bool {
        let (key, bit) = x.split();
        if let Some(b) = self.blocks.get_mut(&key) {
            b.remove(bit)
        } else {
            false
        }
    }

    pub fn iter<'r>(&'r self) -> impl Iterator<Item = u32> + 'r
    where
        'a: 'r,
    {
        self.blocks.iter().flat_map(|(&key, block)| {
            block.iter().map(
                move |val| <u32 as Merge>::merge((key, val)),
            )
        })
    }
}

impl<'a> ::std::ops::Index<u32> for BitVec<'a> {
    type Output = bool;
    fn index(&self, i: u32) -> &Self::Output {
        if self.get(i) {
            super::TRUE
        } else {
            super::FALSE
        }
    }
}

impl<'a> ::Rank<u32> for BitVec<'a> {
    type Weight = u64;

    fn size(&self) -> Self::Weight {
        const CAPACITY: u64 = 1 << 32;
        CAPACITY
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
                rank += Self::Weight::from(block.count_ones());
            }
        }
        rank
    }
}

impl<'a> ::Select1<u32> for BitVec<'a> {
    fn select1(&self, c: u32) -> Option<u32> {
        if self.count_ones() <= c as u64 {
            return None;
        }
        let mut rem = c;
        for (&key, b) in &self.blocks {
            let ones = b.count_ones();
            if rem >= ones {
                rem -= ones;
            } else {
                let s = b.select1(rem as u16).unwrap() as u32;
                let k = (key as u32) << 16;
                return Some(k + s);
            }
        }
        None
    }
}

impl<'a> ::Select0<u32> for BitVec<'a> {
    fn select0(&self, c: u32) -> Option<u32> {
        if self.count_zeros() <= c as u64 {
            return None;
        }
        let mut rem = c;
        for (&key, b) in &self.blocks {
            let zeros = b.count_zeros();
            if rem >= zeros {
                rem -= zeros;
            } else {
                let s = b.select0(rem as u16).unwrap() as u32;
                let k = if key == 0 { 0 } else { (key as u32) - 1 << 16 };
                return Some(k + s);
            }
        }
        None
    }
}

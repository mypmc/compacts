mod stats;
mod pairwise;

use std::fmt::{self, Debug, Formatter};
use std::collections::BTreeMap;
use karabiner::thunk::Thunk;
use {Block, Split, Merge};

pub use self::stats::{Stats, BlockStats};

type Lazy<T> = Thunk<'static, T>;

#[derive(Default)]
pub struct BitVec {
    blocks: BTreeMap<u16, Lazy<Block>>,
}

impl Debug for BitVec {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let b = self.count_blocks();
        let w = self.count_ones();
        write!(f, "BitVec{{ blocks={:?} weight={:?} }}", b, w)
    }
}
impl Clone for BitVec {
    fn clone(&self) -> Self {
        let mut vec = BitVec::new();
        for (&k, t) in &self.blocks {
            let c = (**t).clone();
            vec.blocks.insert(k, eval!(c));
        }
        vec
    }
}

impl BitVec {
    pub fn count_ones(&self) -> u64 {
        self.blocks
            .values()
            .map(|b| u64::from(b.count_ones()))
            .sum()
    }

    pub fn count_zeros(&self) -> u64 {
        (1 << 32) - self.count_ones()
    }

    pub fn mem_size(&self) -> u64 {
        let mut sum = 0;
        for size in self.blocks.values().map(|b| b.mem_size() as u64) {
            sum += size;
        }
        sum
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

impl BitVec {
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
    /// bits.insert(0);
    /// assert!(bits.count_ones() == 1);
    /// bits.clear();
    /// assert!(bits.count_ones() == 0);
    /// ```
    pub fn clear(&mut self) {
        self.blocks.clear();
    }

    /// Return `true` if the value exists.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compacts_bits::BitVec;
    ///
    /// let mut bits = BitVec::new();
    /// assert_eq!(bits.count_zeros(), 1 << 32);
    /// bits.insert(1);
    /// assert!(!bits.contains(0));
    /// assert!(bits.contains(1));
    /// assert!(!bits.contains(2));
    /// assert_eq!(bits.count_ones(), 1);
    /// ```
    pub fn contains(&self, x: u32) -> bool {
        let (key, bit) = x.split();
        self.blocks.get(&key).map_or(false, |b| b.contains(bit))
    }

    /// Return `true` if the value doesn't exists and inserted successfuly.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compacts_bits::BitVec;
    /// let mut bits = BitVec::new();
    /// assert!(bits.insert(3));
    /// assert!(!bits.insert(3));
    /// assert!(bits.contains(3));
    /// assert_eq!(bits.count_ones(), 1);
    /// ```
    pub fn insert(&mut self, x: u32) -> bool {
        let (key, bit) = x.split();
        let mut b = self.blocks
            .entry(key)
            .or_insert_with(|| eval!(Block::new()));
        b.insert(bit)
    }

    /// Return `true` if the value exists and removed successfuly.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compacts_bits::BitVec;
    /// let mut bits = BitVec::new();
    /// assert!(bits.insert(3));
    /// assert!(bits.remove(3));
    /// assert!(!bits.contains(3));
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

    pub fn iter<'r>(&'r self) -> impl Iterator<Item = u32> + 'r {
        self.blocks.iter().flat_map(|(&key, block)| {
            block
                .iter()
                .map(move |val| <u32 as Merge>::merge((key, val)))
        })
    }
}

impl ::std::ops::Index<u32> for BitVec {
    type Output = bool;
    fn index(&self, i: u32) -> &Self::Output {
        if self.contains(i) {
            super::TRUE
        } else {
            super::FALSE
        }
    }
}

impl ::Rank<u32> for BitVec {
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

impl ::Select1<u32> for BitVec {
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

impl ::Select0<u32> for BitVec {
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
                let k = if key == 0 { 0 } else { (key as u32 - 1) << 16 };
                return Some(k + s);
            }
        }
        None
    }
}

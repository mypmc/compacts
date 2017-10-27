mod io;
mod repr;
mod pair;

use std::{iter, ops, slice};
use std::fmt::{self, Debug, Formatter};
use std::borrow::Cow;
use bits::{self, Merge, Split};
use bits::{PopCount, Rank, Select0, Select1};

use self::pair::{Assign, Compare};

pub use self::pair::{Entry, Pair};
pub use self::pair::{And, AndNot, Or, Xor};
pub use self::pair::{and, and_not, or, xor};
pub(crate) use self::repr::Repr;
pub(crate) use self::repr::{Arr64, Run16, Seq16};

/// Set of u32.
#[derive(Clone, Default)]
pub struct Set {
    blocks: Vec<Block>,
}
#[derive(Clone, Default)]
struct Block {
    slot: u16,
    repr: Repr,
}

impl Debug for Set {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let b = self.blocks.len();
        write!(f, "Set {{ {:?} }}", b)
    }
}

impl Set {
    /// Return new Set.
    pub fn new() -> Self {
        Set { blocks: Vec::new() }
    }

    fn search(&self, key: u16) -> Result<usize, usize> {
        self.blocks.binary_search_by_key(&key, |block| block.slot)
    }

    /// Clear contents from set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// #[macro_use]
    /// extern crate compacts;
    /// fn main() {
    ///     use compacts::bits::PopCount;
    ///     let mut bits = bitset!(0);
    ///     assert!(bits.count1() == 1);
    ///     bits.clear();
    ///     assert!(bits.count1() == 0);
    /// }
    /// ```
    pub fn clear(&mut self) {
        self.blocks.clear();
    }

    /// Set flag at `x`, and return a **previous** value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// #[macro_use]
    /// extern crate compacts;
    /// fn main() {
    ///     let mut bits = bitset![1, 2, 8];
    ///     assert!(!bits.set(0, false));
    ///     assert!(bits.set(1, false));
    ///     assert!(!bits.set(1, true));
    ///     assert!(bits.set(1, true));
    /// }
    /// ```
    pub fn set(&mut self, x: u32, flag: bool) -> bool {
        if flag {
            !self.insert(x)
        } else {
            self.remove(x)
        }
    }

    /// Return `true` if `x` exists.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compacts::bits::{Set, PopCount};
    ///
    /// let mut bits = Set::new();
    /// assert_eq!(bits.count0(), 1 << 32);
    /// bits.insert(1);
    /// assert!(!bits.contains(0));
    /// assert!(bits.contains(1));
    /// assert!(!bits.contains(2));
    /// assert_eq!(bits.count1(), 1);
    /// ```
    pub fn contains(&self, x: u32) -> bool {
        let (slot, bit) = x.split();
        self.search(slot)
            .map(|i| self.blocks[i].repr.contains(bit))
            .unwrap_or(false)
    }

    /// Equivalent to `!set(x, true)`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compacts::bits::{Set, PopCount};
    ///
    /// let mut bits = Set::new();
    /// assert!(bits.insert(3));
    /// assert!(!bits.insert(3));
    /// assert!(bits.contains(3));
    /// assert_eq!(bits.count1(), 1);
    /// ```
    pub fn insert(&mut self, x: u32) -> bool {
        let (slot, bit) = x.split();
        let pos = self.search(slot);
        match pos {
            Ok(i) => {
                let block = &mut self.blocks[i];
                block.repr.insert(bit)
            }
            Err(i) => {
                let mut repr = Repr::new();
                repr.insert(bit);
                self.blocks.insert(i, Block { slot, repr });
                true
            }
        }
    }

    /// Equivalent to `set(x, false)`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compacts::bits::{Set, PopCount};
    ///
    /// let mut bits = Set::new();
    /// assert!(bits.insert(3));
    /// assert!(bits.remove(3));
    /// assert!(!bits.contains(3));
    /// assert_eq!(bits.count1(), 0);
    /// ```
    pub fn remove(&mut self, x: u32) -> bool {
        let (slot, bit) = x.split();
        let pos = self.search(slot);
        match pos {
            Ok(i) => {
                let block = &mut self.blocks[i];
                block.repr.remove(bit)
            }
            Err(_) => false,
        }
    }

    /// Optimize innternal data representaions.
    pub fn optimize(&mut self) {
        for block in &mut self.blocks {
            block.repr.optimize();
        }
        self.blocks.retain(|block| block.repr.count1() > 0);
        self.blocks.shrink_to_fit();
    }

    // pub fn mem_size(&self) -> usize {
    //     self.dat.values().map(|b| b.mem_size()).sum()
    // }

    fn blocks(&self) -> Blocks {
        Blocks(self.blocks.iter().map(to_entry))
    }

    pub fn bits<'a>(&'a self) -> impl Iterator<Item = u32> + 'a {
        self.blocks.iter().flat_map(|block| {
            let slot = block.slot;
            block
                .repr
                .iter()
                .map(move |val| <u32 as Merge>::merge((slot, val)))
        })
    }
}

type ToEntry = for<'x> fn(&'x Block) -> bits::Entry<'x>;

pub struct Blocks<'a>(iter::Map<slice::Iter<'a, Block>, ToEntry>);

fn to_entry(block: &Block) -> bits::Entry {
    let key = block.slot;
    let cow = Cow::Borrowed(&block.repr);
    bits::Entry { key, cow }
}

impl<'a> IntoIterator for &'a Set {
    type Item = bits::Entry<'a>;
    type IntoIter = Blocks<'a>;
    fn into_iter(self) -> Self::IntoIter {
        self.blocks()
    }
}

impl<'a> Iterator for Blocks<'a> {
    type Item = bits::Entry<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl Set {
    pub fn and<'a, T>(&'a self, that: T) -> bits::And<impl Iterator<Item = bits::Entry<'a>>>
    where
        T: IntoIterator<Item = bits::Entry<'a>>,
    {
        bits::and(self, that)
    }

    pub fn or<'a, T>(&'a self, that: T) -> bits::Or<impl Iterator<Item = bits::Entry<'a>>>
    where
        T: IntoIterator<Item = bits::Entry<'a>>,
    {
        bits::or(self, that)
    }

    pub fn and_not<'a, T>(&'a self, that: T) -> bits::AndNot<impl Iterator<Item = bits::Entry<'a>>>
    where
        T: IntoIterator<Item = bits::Entry<'a>>,
    {
        bits::and_not(self, that)
    }

    pub fn xor<'a, T>(&'a self, that: T) -> bits::Xor<impl Iterator<Item = bits::Entry<'a>>>
    where
        T: IntoIterator<Item = bits::Entry<'a>>,
    {
        bits::xor(self, that)
    }
}

impl ops::Index<u32> for Set {
    type Output = bool;

    /// # Examples
    ///
    /// ```rust
    /// #[macro_use]
    /// extern crate compacts;
    /// fn main() {
    ///     let bits = bitset!(0, 1 << 30);
    ///     assert!(bits[0]);
    ///     assert!(!bits[1 << 10]);
    ///     assert!(!bits[1 << 20]);
    ///     assert!(bits[1 << 30]);
    /// }
    /// ```
    fn index(&self, i: u32) -> &Self::Output {
        if self.contains(i) {
            super::TRUE
        } else {
            super::FALSE
        }
    }
}

impl<T: AsRef<[u32]>> From<T> for Set {
    fn from(v: T) -> Self {
        v.as_ref().iter().collect()
    }
}

impl<'a> iter::FromIterator<u32> for Set {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = u32>,
    {
        let mut bs = Set::new();
        for b in iter {
            bs.insert(b);
        }
        bs.optimize();
        bs
    }
}

impl<'a> iter::FromIterator<&'a u32> for Set {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a u32>,
    {
        let mut bs = Set::new();
        for b in iter {
            bs.insert(*b);
        }
        bs.optimize();
        bs
    }
}

impl<'a> iter::FromIterator<bits::Entry<'a>> for Set {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = bits::Entry<'a>>,
    {
        let mut blocks = Vec::new();
        for e in iter {
            let slot = e.key;
            let repr = {
                let mut repr = e.cow.into_owned();
                repr.optimize();
                repr
            };
            blocks.push(Block { slot, repr });
        }

        // assume I is sorted by key and all keys are unique.

        Set { blocks }
    }
}

impl PopCount<u64> for Set {
    const SIZE: u64 = 1 << 32;

    /// # Examples
    ///
    /// ```rust
    /// #[macro_use]
    /// extern crate compacts;
    /// fn main() {
    ///     use compacts::bits::PopCount;
    ///     let bits = bitset![0, 1, 4, 1 << 8, 1 << 16];
    ///     assert_eq!(bits.count1(), 5);
    /// }
    /// ```
    fn count1(&self) -> u64 {
        self.blocks.iter().map(|b| u64::from(b.repr.count1())).sum()
    }
}

impl Rank<u32> for Set {
    /// # Examples
    ///
    /// ```rust
    /// #[macro_use]
    /// extern crate compacts;
    /// fn main() {
    ///     use compacts::bits::Rank;
    ///     let bits = bitset![0, 1, 4, 1 << 8, 1 << 16];
    ///     assert_eq!(bits.rank1(0), 0);
    ///     assert_eq!(bits.rank1(1), 1);
    ///     assert_eq!(bits.rank1(2), 2);
    ///     assert_eq!(bits.rank1(3), 2);
    ///     assert_eq!(bits.rank1(4), 2);
    ///     assert_eq!(bits.rank1(5), 3);
    /// }
    /// ```
    fn rank1(&self, i: u32) -> u32 {
        let (hi, lo) = i.split();
        let mut rank = 0;
        for block in &self.blocks {
            if block.slot > hi {
                break;
            } else if block.slot == hi {
                rank += u32::from(block.repr.rank1(lo));
                break;
            } else {
                rank += block.repr.count1();
            }
        }
        rank
    }
}

impl Select1<u32> for Set {
    /// # Examples
    ///
    /// ```rust
    /// #[macro_use]
    /// extern crate compacts;
    /// fn main() {
    ///     use compacts::bits::Select1;
    ///     let bits = bitset![0, 1, 4, 1 << 8, 1 << 16];
    ///     assert_eq!(bits.select1(0), Some(0));
    ///     assert_eq!(bits.select1(1), Some(1));
    ///     assert_eq!(bits.select1(2), Some(4));
    ///     assert_eq!(bits.select1(3), Some(1 << 8));
    /// }
    /// ```
    fn select1(&self, c: u32) -> Option<u32> {
        if self.count1() <= u64::from(c) {
            return None;
        }
        let mut remain = c;
        for block in &self.blocks {
            let w = block.repr.count1();
            if remain >= w {
                remain -= w;
            } else {
                let s = u32::from(block.repr.select1(remain as u16).unwrap());
                let k = u32::from(block.slot) << 16;
                return Some(s + k);
            }
        }
        None
    }
}

impl Select0<u32> for Set {
    /// # Examples
    ///
    /// ```rust
    /// #[macro_use]
    /// extern crate compacts;
    /// fn main() {
    ///     use compacts::bits::Select0;
    ///     let bits = bitset![0, 1, 4, 1 << 8, 1 << 16];
    ///     assert_eq!(bits.select0(0), Some(2));
    ///     assert_eq!(bits.select0(1), Some(3));
    ///     assert_eq!(bits.select0(2), Some(5));
    ///     assert_eq!(bits.select0(3), Some(6));
    /// }
    /// ```
    fn select0(&self, c: u32) -> Option<u32> {
        if self.count0() <= u64::from(c) {
            return None;
        }
        select_by_rank!(0, self, c, 0u64, 1 << 32, u32)
    }
}

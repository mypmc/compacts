mod io;

use std::{iter, ops, slice};
use std::fmt::{self, Debug, Formatter};
use std::borrow::Cow;

use bits::{self, Block, Merge, Split};
use bits::{PopCount, Rank, Select0, Select1};

/// Set of `Block`s.
#[derive(Clone, Default)]
pub struct Set {
    entries: Vec<Keyed>,
}
#[derive(Clone, Default)]
pub struct Keyed {
    key: u16,
    block: Block,
}

impl Debug for Set {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let b = self.entries.len();
        write!(f, "Set {{ {:?} }}", b)
    }
}

impl Set {
    pub fn new() -> Self {
        Set {
            entries: Vec::new(),
        }
    }

    fn search(&self, key: u16) -> Result<usize, usize> {
        self.entries.binary_search_by_key(&key, |ref e| e.key)
    }

    /// Clear contents.
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
        self.entries.clear();
    }

    /// Return `true` if the value exists.
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
        let (key, bit) = x.split();
        self.search(key)
            .map(|i| self.entries[i].block.contains(bit))
            .unwrap_or(false)
    }

    /// Return `true` if the value doesn't exists and inserted successfuly.
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
        let (key, bit) = x.split();
        let pos = self.search(key);
        match pos {
            Ok(i) => {
                let mut e = self.entries.get_mut(i).unwrap();
                e.block.insert(bit)
            }
            Err(i) => {
                let mut block = Block::new();
                block.insert(bit);
                self.entries.insert(i, Keyed { key, block });
                true
            }
        }
    }

    /// Return `true` if the value exists and removed successfuly.
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
        let (key, bit) = x.split();
        let pos = self.search(key);
        match pos {
            Ok(i) => {
                let mut e = self.entries.get_mut(i).unwrap();
                e.block.remove(bit)
            }
            Err(_) => false,
        }
    }

    /// Optimize innternal data representaions.
    pub fn optimize(&mut self) {
        for keyed in &mut self.entries {
            keyed.block.optimize();
        }
        self.entries.retain(|ref e| e.block.count1() > 0);
        self.entries.shrink_to_fit();
    }

    // pub fn mem_size(&self) -> usize {
    //     self.blocks.values().map(|b| b.mem_size()).sum()
    // }

    // // pub fn stats<'a>(&'a self) -> impl Iterator<Item = block::Stats> + 'a {
    // //     self.blocks.values().map(|b| b.stats())
    // // }

    pub fn entries(&self) -> Entries {
        Entries(self.entries.iter().map(to_entry))
    }

    pub fn bits<'a>(&'a self) -> impl Iterator<Item = u32> + 'a {
        self.entries.iter().flat_map(|ref keyed| {
            let key = keyed.key;
            keyed
                .block
                .iter()
                .map(move |val| <u32 as Merge>::merge((key, val)))
        })
    }
}

type ToEntry = for<'x> fn(&'x Keyed) -> bits::Entry<'x>;

pub struct Entries<'a>(iter::Map<slice::Iter<'a, Keyed>, ToEntry>);

fn to_entry<'a>(keyed: &'a Keyed) -> bits::Entry<'a> {
    let key = keyed.key;
    let cow = Cow::Borrowed(&keyed.block);
    bits::Entry { key, cow }
}

impl<'a> IntoIterator for &'a Set {
    type Item = bits::Entry<'a>;
    type IntoIter = Entries<'a>;
    fn into_iter(self) -> Self::IntoIter {
        self.entries()
    }
}

impl<'a> Iterator for Entries<'a> {
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
        let mut entries = Vec::new();
        for e in iter {
            let key = e.key;
            let block = {
                let mut block = e.cow.into_owned();
                block.optimize();
                block
            };
            entries.push(Keyed { key, block });
        }

        // assume I is sorted by key and all keys are unique.

        Set { entries }
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
        self.entries
            .iter()
            .map(|e| u64::from(e.block.count1()))
            .sum()
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
        for e in &self.entries {
            if e.key > hi {
                break;
            } else if e.key == hi {
                rank += u32::from(e.block.rank1(lo));
                break;
            } else {
                rank += u32::from(e.block.count1());
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
        for e in &self.entries {
            let w = e.block.count1();
            if remain >= w {
                remain -= w;
            } else {
                let s = u32::from(e.block.select1(remain as u16).unwrap());
                let k = u32::from(e.key) << 16;
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

// mod io;

use std::{iter, ops};
use std::fmt::{self, Debug, Formatter};
use std::collections::{btree_map, BTreeMap};
use std::borrow::Cow;
use std::iter::IntoIterator;

use bits::{self, Block, Merge, Split};
use bits::{PopCount, Rank, Select0, Select1};

/// Map of deffered `Block`s.
#[derive(Default)]
pub struct Map {
    blocks: BTreeMap<u16, Block>,
}

impl Debug for Map {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let b = self.blocks.len();
        write!(f, "Map {{ blocks:{:?} }}", b)
    }
}
impl Clone for Map {
    fn clone(&self) -> Self {
        let mut map = Map::new();
        for (&k, block) in &self.blocks {
            map.blocks.insert(k, block.clone());
        }
        map
    }
}

impl Map {
    pub fn new() -> Self {
        Map {
            blocks: BTreeMap::new(),
        }
    }

    /// Clear contents.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compacts::bits::{Map, PopCount};
    ///
    /// let mut bits = Map::new();
    /// bits.insert(0);
    /// assert!(bits.count1() == 1);
    /// bits.clear();
    /// assert!(bits.count1() == 0);
    /// ```
    pub fn clear(&mut self) {
        self.blocks.clear();
    }

    /// Return `true` if the value exists.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compacts::bits::{Map, PopCount};
    ///
    /// let mut bits = Map::new();
    /// assert_eq!(bits.count0(), 1 << 32);
    /// bits.insert(1);
    /// assert!(!bits.contains(0));
    /// assert!(bits.contains(1));
    /// assert!(!bits.contains(2));
    /// assert_eq!(bits.count1(), 1);
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
    /// use compacts::bits::{Map, PopCount};
    ///
    /// let mut bits = Map::new();
    /// assert!(bits.insert(3));
    /// assert!(!bits.insert(3));
    /// assert!(bits.contains(3));
    /// assert_eq!(bits.count1(), 1);
    /// ```
    pub fn insert(&mut self, x: u32) -> bool {
        let (key, bit) = x.split();
        let b = self.blocks.entry(key).or_insert_with(Block::new);
        b.insert(bit)
    }

    /// Return `true` if the value exists and removed successfuly.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compacts::bits::{Map, PopCount};
    ///
    /// let mut bits = Map::new();
    /// assert!(bits.insert(3));
    /// assert!(bits.remove(3));
    /// assert!(!bits.contains(3));
    /// assert_eq!(bits.count1(), 0);
    /// ```
    pub fn remove(&mut self, x: u32) -> bool {
        let (key, bit) = x.split();
        if let Some(b) = self.blocks.get_mut(&key) {
            b.remove(bit)
        } else {
            false
        }
    }

    /// Optimize innternal data representaions.
    pub fn optimize(&mut self) {
        let mut remove_keys = Vec::new();
        for (k, b) in &mut self.blocks {
            b.optimize();
            if b.count1() == 0 {
                remove_keys.push(*k)
            }
        }
        for key in remove_keys {
            self.blocks.remove(&key);
        }
    }

    pub fn mem_size(&self) -> usize {
        self.blocks.values().map(|b| b.mem_size()).sum()
    }

    // pub fn stats<'a>(&'a self) -> impl Iterator<Item = block::Stats> + 'a {
    //     self.blocks.values().map(|b| b.stats())
    // }

    pub fn entries<'a>(&'a self) -> Entries<'a> {
        Entries(self.blocks.iter().map(to_entry))
    }

    pub fn bits<'a>(&'a self) -> impl Iterator<Item = u32> + 'a {
        self.blocks.iter().flat_map(|(&key, block)| {
            block
                .iter()
                .map(move |val| <u32 as Merge>::merge((key, val)))
        })
    }
}

pub struct Entries<'a>(
    iter::Map<
        btree_map::Iter<'a, u16, bits::Block>,
        for<'x> fn((&'x u16, &'x bits::Block)) -> bits::Entry<'x>,
    >,
);

fn to_entry<'a>(tuple: (&'a u16, &'a bits::Block)) -> bits::Entry<'a> {
    let (&key, block) = tuple;
    let cow = Cow::Borrowed(block);
    bits::Entry { key, cow }
}

impl<'a> IntoIterator for &'a bits::Map {
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

impl Map {
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

impl ops::Index<u32> for Map {
    type Output = bool;

    /// # Examples
    ///
    /// ```rust
    /// use compacts::bits::Map;
    /// let bits = Map::from(vec![0, 1 << 30]);
    /// assert!(bits[0]);
    /// assert!(!bits[1 << 10]);
    /// assert!(!bits[1 << 20]);
    /// assert!(bits[1 << 30]);
    /// ```
    fn index(&self, i: u32) -> &Self::Output {
        if self.contains(i) {
            super::TRUE
        } else {
            super::FALSE
        }
    }
}

impl<T: AsRef<[u32]>> From<T> for Map {
    fn from(v: T) -> Self {
        v.as_ref().iter().collect()
    }
}

impl<'a> iter::FromIterator<u32> for Map {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = u32>,
    {
        let mut map = Map::new();
        for b in iter {
            map.insert(b);
        }
        map.optimize();
        map
    }
}

impl<'a> iter::FromIterator<&'a u32> for Map {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a u32>,
    {
        let mut map = Map::new();
        for b in iter {
            map.insert(*b);
        }
        map.optimize();
        map
    }
}

impl<'a> iter::FromIterator<bits::Entry<'a>> for Map {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = bits::Entry<'a>>,
    {
        let mut blocks = BTreeMap::new();
        for e in iter {
            let mut b = e.cow.into_owned();
            b.optimize();
            blocks.insert(e.key, b);
        }
        Map { blocks }
    }
}

impl PopCount<u64> for Map {
    const SIZE: u64 = 1 << 32;

    /// # Examples
    ///
    /// ```rust
    /// use compacts::bits::{Map, PopCount};
    /// let bits = Map::from(vec![0, 1, 4, 1 << 8, 1 << 16]);
    /// assert_eq!(bits.count1(), 5);
    /// ```
    fn count1(&self) -> u64 {
        self.blocks.values().map(|b| u64::from(b.count1())).sum()
    }
}

impl Rank<u32> for Map {
    /// # Examples
    ///
    /// ```rust
    /// use compacts::bits::{Map, Rank};
    /// let bits = Map::from(vec![0, 1, 4, 1 << 8, 1 << 16]);
    /// assert_eq!(bits.rank1(0), 0);
    /// assert_eq!(bits.rank1(1), 1);
    /// assert_eq!(bits.rank1(2), 2);
    /// assert_eq!(bits.rank1(3), 2);
    /// assert_eq!(bits.rank1(4), 2);
    /// assert_eq!(bits.rank1(5), 3);
    /// ```
    fn rank1(&self, i: u32) -> u32 {
        let (hi, lo) = i.split();
        let mut rank = 0;
        for (&key, block) in &self.blocks {
            if key > hi {
                break;
            } else if key == hi {
                rank += u32::from(block.rank1(lo));
                break;
            } else {
                rank += u32::from(block.count1());
            }
        }
        rank
    }
}

impl Select1<u32> for Map {
    /// # Examples
    ///
    /// ```rust
    /// use compacts::bits::{Map, Select1};
    /// let bits = Map::from(vec![0, 1, 4, 1 << 8, 1 << 16]);
    /// assert_eq!(bits.select1(0), Some(0));
    /// assert_eq!(bits.select1(1), Some(1));
    /// assert_eq!(bits.select1(2), Some(4));
    /// assert_eq!(bits.select1(3), Some(1 << 8));
    /// ```
    fn select1(&self, c: u32) -> Option<u32> {
        if self.count1() <= u64::from(c) {
            return None;
        }
        let mut remain = c;
        for (&key, b) in &self.blocks {
            let w = b.count1();
            if remain >= w {
                remain -= w;
            } else {
                let s = u32::from(b.select1(remain as u16).unwrap());
                let k = u32::from(key) << 16;
                return Some(s + k);
            }
        }
        None
    }
}

impl Select0<u32> for Map {
    /// # Examples
    ///
    /// ```rust
    /// use compacts::bits::{Map, Select0};
    /// let bits = Map::from(vec![0, 1, 4, 1 << 8, 1 << 16]);
    /// assert_eq!(bits.select0(0), Some(2));
    /// assert_eq!(bits.select0(1), Some(3));
    /// assert_eq!(bits.select0(2), Some(5));
    /// assert_eq!(bits.select0(3), Some(6));
    /// ```
    fn select0(&self, c: u32) -> Option<u32> {
        if self.count0() <= u64::from(c) {
            return None;
        }
        select_by_rank!(0, self, c, 0u64, 1 << 32, u32)
    }
}

use std::collections::BTreeMap;
use std::{iter, ops};

use bits::Map32;
use bits::prim::{Merge, Split};
use bits::pair::*;
use bits::block;
use dict::{PopCount, Rank, Select0, Select1};

/// Map of Map32.
#[derive(Clone, Debug)]
pub struct Map64 {
    map32s: BTreeMap<u32, Map32>,
}

impl Default for Map64 {
    fn default() -> Self {
        let map32s = BTreeMap::new();
        Map64 { map32s }
    }
}

impl Map64 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.map32s.clear()
    }

    /// Return `true` if the value exists.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compacts::bits::Map64;
    /// let mut bits = Map64::new();
    /// assert!(!bits.contains(1 << 50));
    /// bits.insert(1 << 50);
    /// assert!(bits.contains(1 << 50));
    /// ```
    pub fn contains(&self, x: u64) -> bool {
        let (key, bit) = x.split();
        self.map32s.get(&key).map_or(false, |b| b.contains(bit))
    }

    /// Return `true` if the value doesn't exists and inserted successfuly.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compacts::bits::Map64;
    /// let mut bits = Map64::new();
    /// assert!(bits.insert(1 << 50));
    /// assert!(!bits.insert(1 << 50));
    /// ```
    pub fn insert(&mut self, x: u64) -> bool {
        let (key, bit) = x.split();
        let b = self.map32s.entry(key).or_insert_with(Map32::new);
        b.insert(bit)
    }

    /// Return `true` if the value exists and removed successfuly.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compacts::bits::Map64;
    /// let mut bits = Map64::new();
    /// assert!(bits.insert(1 << 60));
    /// assert!(bits.remove(1 << 60));
    /// ```
    pub fn remove(&mut self, x: u64) -> bool {
        let (key, bit) = x.split();
        self.map32s.get_mut(&key).map_or(false, |b| b.remove(bit))
    }

    pub fn iter<'r>(&'r self) -> impl Iterator<Item = u64> + 'r {
        self.map32s.iter().flat_map(|(&key, vec)| {
            vec.iter().map(move |val| <u64 as Merge>::merge((key, val)))
        })
    }

    pub fn mem_size(&self) -> usize {
        self.map32s.values().map(|b| b.mem_size()).sum()
    }

    pub fn stats<'r>(&'r self) -> impl Iterator<Item = block::Stats> + 'r {
        self.map32s.values().flat_map(|vec| vec.stats())
    }

    /// Optimize innternal data representaions.
    pub fn optimize(&mut self) {
        let mut remove_keys = Vec::new();
        for (k, vec) in &mut self.map32s {
            vec.optimize();
            if vec.count1() == 0 {
                remove_keys.push(*k);
            }
        }
        for key in remove_keys {
            self.map32s.remove(&key);
        }
    }
}

impl ops::Index<u64> for Map64 {
    type Output = bool;

    /// # Examples
    ///
    /// ```rust
    /// use compacts::bits::Map64;
    /// let bits = Map64::from(vec![0, 1 << 60]);
    /// assert!(bits[0]);
    /// assert!(!bits[1 << 20]);
    /// assert!(!bits[1 << 30]);
    /// assert!(bits[1 << 60]);
    /// ```
    fn index(&self, i: u64) -> &Self::Output {
        if self.contains(i) {
            super::TRUE
        } else {
            super::FALSE
        }
    }
}

impl<'a> iter::FromIterator<u64> for Map64 {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = u64>,
    {
        let mut vec = Map64::new();
        for b in iter {
            vec.insert(b);
        }
        vec.optimize();
        vec
    }
}

impl<'a> iter::FromIterator<&'a u64> for Map64 {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a u64>,
    {
        let mut vec = Map64::new();
        for b in iter {
            vec.insert(*b);
        }
        vec.optimize();
        vec
    }
}

impl<T: AsRef<[u64]>> From<T> for Map64 {
    fn from(v: T) -> Self {
        v.as_ref().iter().collect()
    }
}

impl PopCount<u128> for Map64 {
    const SIZE: u128 = 1 << 64;

    /// # Examples
    ///
    /// ```rust
    /// use compacts::bits::Map64;
    /// use compacts::dict::PopCount;
    /// let bits = Map64::from(vec![0, 1, 4, 1 << 32, 1 << 50, 1 << 60]);
    /// assert_eq!(bits.count1(), 6);
    /// ```
    fn count1(&self) -> u128 {
        self.map32s.values().map(|b| u128::from(b.count1())).sum()
    }
}


impl Rank<u64> for Map64 {
    /// # Examples
    ///
    /// ```rust
    /// use compacts::bits::Map64;
    /// use compacts::dict::Rank;
    /// let bits = Map64::from(vec![0, 1, 4, 1 << 32, 1 << 50, 1 << 60]);
    /// assert_eq!(bits.rank1(0), 0);
    /// assert_eq!(bits.rank1(1), 1);
    /// assert_eq!(bits.rank1(2), 2);
    /// assert_eq!(bits.rank1(3), 2);
    /// assert_eq!(bits.rank1(4), 2);
    /// assert_eq!(bits.rank1(5), 3);
    /// ```
    fn rank1(&self, i: u64) -> u64 {
        let (hi, lo) = i.split();
        let mut rank = 0;
        for (&key, vec) in &self.map32s {
            if key > hi {
                break;
            } else if key == hi {
                rank += u64::from(vec.rank1(lo));
                break;
            } else {
                rank += u64::from(vec.count1());
            }
        }
        rank
    }
}

impl Select1<u64> for Map64 {
    /// # Examples
    ///
    /// ```rust
    /// use compacts::bits::Map64;
    /// use compacts::dict::Select1;
    /// let bits = Map64::from(vec![0, 1, 4, 1 << 32, 1 << 50, 1 << 60]);
    /// assert_eq!(bits.select1(0), Some(0));
    /// assert_eq!(bits.select1(1), Some(1));
    /// assert_eq!(bits.select1(2), Some(4));
    /// assert_eq!(bits.select1(3), Some(1 << 32));
    /// ```
    fn select1(&self, c: u64) -> Option<u64> {
        if self.count1() <= u128::from(c) {
            return None;
        }
        let mut remain = c;
        for (&key, b) in &self.map32s {
            let w = b.count1();
            if remain >= w {
                remain -= w;
            } else {
                let s = u64::from(b.select1(remain as u32).unwrap());
                let k = u64::from(key) << 32;
                return Some(k + s);
            }
        }
        None
    }
}

impl Select0<u64> for Map64 {
    /// # Examples
    ///
    /// ```rust
    /// use compacts::bits::Map64;
    /// use compacts::dict::Select0;
    /// let bits = Map64::from(vec![0, 1, 4, 1 << 32, 1 << 50, 1 << 60]);
    /// assert_eq!(bits.select0(0), Some(2));
    /// assert_eq!(bits.select0(1), Some(3));
    /// assert_eq!(bits.select0(2), Some(5));
    /// assert_eq!(bits.select0(3), Some(6));
    /// ```
    fn select0(&self, c: u64) -> Option<u64> {
        if self.count0() <= u128::from(c) {
            return None;
        }
        select_by_rank!(0, self, c, 0u128, 1 << 64, u64)
    }
}

macro_rules! impl_Pairwise {
    ( $( ( $op:ident, $fn:ident, $fn_with:ident ) ),* ) => ($(
        impl $op<Map64> for Map64 {
            type Output = Map64;
            fn $fn(self, that: Map64) -> Self::Output {
                let mut this = self;
                this.$fn_with(&that);
                this
            }
        }
        impl<'r> $op<&'r Map64> for Map64 {
            type Output = Map64;
            fn $fn(self, that: &Map64) -> Self::Output {
                let mut this = self;
                this.$fn_with(that);
                this
            }
        }
        impl<'r1, 'r2> $op<&'r2 Map64> for &'r1 Map64 {
            type Output = Map64;
            fn $fn(self, that: &Map64) -> Self::Output {
                let mut this = self.clone();
                this.$fn_with(that);
                this
            }
        }
    )*)
}

impl_Pairwise!(
    (Intersection, intersection, intersection_with),
    (Union, union, union_with),
    (Difference, difference, difference_with),
    (
        SymmetricDifference,
        symmetric_difference,
        symmetric_difference_with
    )
);

impl<'r> IntersectionWith<&'r Map64> for Map64 {
    fn intersection_with(&mut self, that: &'r Map64) {
        let keys_to_remove = {
            let mut keys = Vec::with_capacity(self.map32s.len());
            for (key, vec) in &mut self.map32s {
                if that.map32s.contains_key(key) {
                    vec.intersection_with(&that.map32s[key]);
                    if vec.count1() != 0 {
                        vec.optimize();
                    } else {
                        keys.push(*key);
                    }
                } else {
                    keys.push(*key);
                }
            }
            keys
        };

        for key in keys_to_remove {
            let removed = self.map32s.remove(&key);
            debug_assert!(removed.is_some());
        }
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(map_entry))]
impl<'r> UnionWith<&'r Map64> for Map64 {
    fn union_with(&mut self, that: &'r Map64) {
        for (&key, vec) in &that.map32s {
            if !self.map32s.contains_key(&key) {
                self.map32s.insert(key, vec.clone());
                continue;
            }
            let mut lb = self.map32s[&key].clone();
            lb.union_with(vec);
            self.map32s.insert(key, lb);
        }
    }
}

impl<'r> DifferenceWith<&'r Map64> for Map64 {
    fn difference_with(&mut self, that: &'r Map64) {
        for (&key, vec) in &mut self.map32s {
            if !that.map32s.contains_key(&key) {
                continue;
            }
            let lb = vec.clone();
            let rb = &that.map32s[&key];
            *vec = lb.difference(rb);
        }
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(map_entry))]
impl<'r> SymmetricDifferenceWith<&'r Map64> for Map64 {
    fn symmetric_difference_with(&mut self, that: &'r Map64) {
        for (&key, vec) in &that.map32s {
            if !self.map32s.contains_key(&key) {
                self.map32s.insert(key, vec.clone());
                continue;
            }
            let mut b = self.map32s[&key].clone();
            b.symmetric_difference_with(vec);
            self.map32s.insert(key, b);
        }
    }
}

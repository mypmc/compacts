use std::collections::BTreeMap;
use {Map32, Rank, Select0, Select1};
use prim::{Merge, Split};
use pair::*;
use block;

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

    pub fn count_ones(&self) -> u128 {
        let mut r = 0;
        for w in self.map32s.iter().map(|(_, vec)| vec.count_ones() as u128) {
            r += w;
        }
        r
    }

    pub fn count_zeros(&self) -> u128 {
        (1 << 64) - self.count_ones()
    }

    pub fn mem_size(&self) -> u128 {
        let mut sum = 0;
        for mem in self.map32s.values().map(|vec| vec.mem_size() as u128) {
            sum += mem;
        }
        sum
    }

    pub fn optimize(&mut self) {
        let mut rs = Vec::new();
        for (k, vec) in &mut self.map32s {
            vec.optimize();
            if vec.count_ones() == 0 {
                rs.push(*k);
            }
        }
        for k in rs {
            self.map32s.remove(&k);
        }
    }

    /// Return `true` if the value exists.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compacts_bits::Map64;
    /// let mut bits = Map64::new();
    /// assert!(!bits.contains(1 << 50));
    /// bits.insert(1 << 50);
    /// assert!(bits.contains(1 << 50));
    /// assert_eq!(1, bits.count_ones());
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
    /// use compacts_bits::Map64;
    /// let mut bits = Map64::new();
    /// assert!(bits.insert(1 << 50));
    /// assert!(!bits.insert(1 << 50));
    /// assert_eq!(1, bits.count_ones());
    /// ```
    pub fn insert(&mut self, x: u64) -> bool {
        let (key, bit) = x.split();
        let mut bv = self.map32s.entry(key).or_insert_with(Map32::new);
        bv.insert(bit)
    }

    /// Return `true` if the value exists and removed successfuly.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compacts_bits::Map64;
    /// let mut bits = Map64::new();
    /// assert!(bits.insert(1 << 60));
    /// assert!(bits.remove(1 << 60));
    /// assert_eq!(0, bits.count_ones());
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

    pub fn stats<'r>(&'r self) -> impl Iterator<Item = block::Stats> + 'r {
        self.map32s.values().flat_map(|vec| vec.stats())
    }

    pub fn summary(&self) -> super::Summary {
        self.stats().sum()
    }
}

impl ::std::ops::Index<u64> for Map64 {
    type Output = bool;
    fn index(&self, i: u64) -> &Self::Output {
        if self.contains(i) {
            super::TRUE
        } else {
            super::FALSE
        }
    }
}

impl<'a> ::std::iter::FromIterator<u64> for Map64 {
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

impl<'a> ::std::iter::FromIterator<&'a u64> for Map64 {
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

impl Rank<u64> for Map64 {
    type Count = u128;

    /// Returns occurences of non-zero bit in `[0,i]`.
    fn rank1(&self, i: u64) -> Self::Count {
        let (hi, lo) = i.split();
        let mut rank = 0;
        for (&key, vec) in &self.map32s {
            if key > hi {
                break;
            } else if key == hi {
                rank += Self::Count::from(vec.rank1(lo));
                break;
            } else {
                rank += Self::Count::from(vec.count_ones());
            }
        }
        rank
    }

    /// Returns occurences of zero bit in `[0,i]`.
    fn rank0(&self, i: u64) -> Self::Count {
        let rank1 = self.rank1(i);
        i as Self::Count + 1 - rank1
    }
}

impl Select1<u64> for Map64 {
    type Index = u64;

    /// Returns the position of 'c+1'th appearance of non-zero bit.
    fn select1(&self, c: u64) -> Option<Self::Index> {
        if self.count_ones() <= c as u128 {
            return None;
        }
        let mut rem = c;
        for (&key, b) in &self.map32s {
            let w = b.count_ones();
            if rem >= w {
                rem -= w;
            } else {
                let s = b.select1(rem as u32).unwrap() as u64;
                let k = (key as u64) << 32;
                return Some(k + s);
            }
        }
        None
    }
}

impl Select0<u64> for Map64 {
    type Index = u64;

    /// Returns the position of 'c+1'th appearance of zero bit.
    fn select0(&self, c: u64) -> Option<Self::Index> {
        if self.count_zeros() <= c as u128 {
            return None;
        }

        let fun = |i| {
            let rank0 = self.rank0(i as u64);
            rank0 > c as u128
        };
        let pos = search!(0u128, 1 << 64, fun);
        if pos < (1 << 64) {
            Some(pos as u64)
        } else {
            None
        }
    }
}

macro_rules! impl_Pairwise {
    ( $( ( $op:ident, $fn:ident, $fn_with:ident ) ),* ) => ($(
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
                    if vec.count_ones() != 0 {
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

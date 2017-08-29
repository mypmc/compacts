use std::{iter, ops};
use std::fmt::{self, Debug, Formatter};
use std::collections::BTreeMap;

use bits::Map16;
use bits::prim::{Merge, Split};
use bits::pair::*;
use bits::{block, thunk};
use dict::{PopCount, Rank, Select0, Select1};

type Lazy<T> = thunk::Thunk<'static, T>;

/// Map of (deffered) Map16.
#[derive(Default)]
pub struct Map32 {
    map16s: BTreeMap<u16, Lazy<Map16>>,
}

impl Debug for Map32 {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let b = self.map16s.len();
        write!(f, "Map32 {{ blocks:{:?} }}", b)
    }
}
impl Clone for Map32 {
    fn clone(&self) -> Self {
        let mut map = Map32::new();
        for (&k, block) in &self.map16s {
            map.map16s.insert(k, eval!((**block).clone()));
        }
        map
    }
}

impl Map32 {
    pub fn new() -> Self {
        Map32 {
            map16s: BTreeMap::new(),
        }
    }

    /// Clear contents.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compacts::bits::Map32;
    /// use compacts::dict::PopCount;
    ///
    /// let mut bits = Map32::new();
    /// bits.insert(0);
    /// assert!(bits.count1() == 1);
    /// bits.clear();
    /// assert!(bits.count1() == 0);
    /// ```
    pub fn clear(&mut self) {
        self.map16s.clear();
    }

    /// Return `true` if the value exists.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compacts::bits::Map32;
    /// use compacts::dict::PopCount;
    ///
    /// let mut bits = Map32::new();
    /// assert_eq!(bits.count0(), 1 << 32);
    /// bits.insert(1);
    /// assert!(!bits.contains(0));
    /// assert!(bits.contains(1));
    /// assert!(!bits.contains(2));
    /// assert_eq!(bits.count1(), 1);
    /// ```
    pub fn contains(&self, x: u32) -> bool {
        let (key, bit) = x.split();
        self.map16s.get(&key).map_or(false, |b| b.contains(bit))
    }

    /// Return `true` if the value doesn't exists and inserted successfuly.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compacts::bits::Map32;
    /// use compacts::dict::PopCount;
    ///
    /// let mut bits = Map32::new();
    /// assert!(bits.insert(3));
    /// assert!(!bits.insert(3));
    /// assert!(bits.contains(3));
    /// assert_eq!(bits.count1(), 1);
    /// ```
    pub fn insert(&mut self, x: u32) -> bool {
        let (key, bit) = x.split();
        let b = self.map16s
            .entry(key)
            .or_insert_with(|| eval!(Map16::new()));
        b.insert(bit)
    }

    /// Return `true` if the value exists and removed successfuly.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compacts::bits::Map32;
    /// use compacts::dict::PopCount;
    ///
    /// let mut bits = Map32::new();
    /// assert!(bits.insert(3));
    /// assert!(bits.remove(3));
    /// assert!(!bits.contains(3));
    /// assert_eq!(bits.count1(), 0);
    /// ```
    pub fn remove(&mut self, x: u32) -> bool {
        let (key, bit) = x.split();
        if let Some(b) = self.map16s.get_mut(&key) {
            b.remove(bit)
        } else {
            false
        }
    }

    pub fn iter<'r>(&'r self) -> impl Iterator<Item = u32> + 'r {
        self.map16s.iter().flat_map(|(&key, block)| {
            block
                .iter()
                .map(move |val| <u32 as Merge>::merge((key, val)))
        })
    }

    pub fn stats<'a>(&'a self) -> impl Iterator<Item = block::Stats> + 'a {
        self.map16s.values().map(|v16| v16.stats())
    }

    pub fn mem_size(&self) -> usize {
        self.map16s.values().map(|b| b.mem_size()).sum()
    }

    /// Optimize innternal data representaions.
    pub fn optimize(&mut self) {
        let mut remove_keys = Vec::new();
        for (k, b) in &mut self.map16s {
            b.optimize();
            if b.count1() == 0 {
                remove_keys.push(*k)
            }
        }
        for key in remove_keys {
            self.map16s.remove(&key);
        }
    }
}

impl ops::Index<u32> for Map32 {
    type Output = bool;

    /// # Examples
    ///
    /// ```rust
    /// use compacts::bits::Map32;
    /// let bits = Map32::from(vec![0, 1 << 30]);
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

impl<T: AsRef<[u32]>> From<T> for Map32 {
    fn from(v: T) -> Self {
        v.as_ref().iter().collect()
    }
}

impl<'a> iter::FromIterator<u32> for Map32 {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = u32>,
    {
        let mut vec = Map32::new();
        for b in iter {
            vec.insert(b);
        }
        vec.optimize();
        vec
    }
}

impl<'a> iter::FromIterator<&'a u32> for Map32 {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a u32>,
    {
        let mut vec = Map32::new();
        for b in iter {
            vec.insert(*b);
        }
        vec.optimize();
        vec
    }
}

impl PopCount<u64> for Map32 {
    const SIZE: u64 = 1 << 32;

    /// # Examples
    ///
    /// ```rust
    /// use compacts::bits::Map32;
    /// use compacts::dict::PopCount;
    /// let bits = Map32::from(vec![0, 1, 4, 1 << 8, 1 << 16]);
    /// assert_eq!(bits.count1(), 5);
    /// ```
    fn count1(&self) -> u64 {
        self.map16s.values().map(|b| u64::from(b.count1())).sum()
    }
}

impl Rank<u32> for Map32 {
    /// # Examples
    ///
    /// ```rust
    /// use compacts::bits::Map32;
    /// use compacts::dict::Rank;
    /// let bits = Map32::from(vec![0, 1, 4, 1 << 8, 1 << 16]);
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
        for (&key, block) in &self.map16s {
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

impl Select1<u32> for Map32 {
    /// # Examples
    ///
    /// ```rust
    /// use compacts::bits::Map32;
    /// use compacts::dict::Select1;
    /// let bits = Map32::from(vec![0, 1, 4, 1 << 8, 1 << 16]);
    /// assert_eq!(bits.select1(0), Some(0));
    /// assert_eq!(bits.select1(1), Some(1));
    /// assert_eq!(bits.select1(2), Some(4));
    /// assert_eq!(bits.select1(3), Some(1 << 8));
    /// ```
    fn select1(&self, c: u32) -> Option<u32> {
        if self.count1() <= c as u64 {
            return None;
        }
        let mut remain = c;
        for (&key, b) in &self.map16s {
            let w = b.count1();
            if remain >= w {
                remain -= w;
            } else {
                let s = b.select1(remain as u16).unwrap() as u32;
                let k = (key as u32) << 16;
                return Some(s + k);
            }
        }
        None
    }
}

impl Select0<u32> for Map32 {
    /// # Examples
    ///
    /// ```rust
    /// use compacts::bits::Map32;
    /// use compacts::dict::Select0;
    /// let bits = Map32::from(vec![0, 1, 4, 1 << 8, 1 << 16]);
    /// assert_eq!(bits.select0(0), Some(2));
    /// assert_eq!(bits.select0(1), Some(3));
    /// assert_eq!(bits.select0(2), Some(5));
    /// assert_eq!(bits.select0(3), Some(6));
    /// ```
    fn select0(&self, c: u32) -> Option<u32> {
        if self.count0() <= c as u64 {
            return None;
        }
        select_by_rank!(0, self, c, 0u64, 1 << 32, u32)
    }
}

macro_rules! impl_Pairwise {
    ( $( ( $op:ident, $fn:ident, $fn_with:ident ) ),* ) => ($(
        impl $op<Map32> for Map32 {
            type Output = Map32;
            fn $fn(self, that: Map32) -> Self::Output {
                let mut this = self;
                this.$fn_with(&that);
                this
            }
        }

        impl<'r> $op<&'r Map32> for Map32 {
            type Output = Map32;
            fn $fn(self, that: &Map32) -> Self::Output {
                let mut this = self;
                this.$fn_with(that);
                this
            }
        }

        impl<'a, 'b> $op<&'b Map32> for &'a Map32 {
            type Output = Map32;
            fn $fn(self, that: &Map32) -> Self::Output {
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

fn pair_union<'a, T: UnionWith<&'a T>>(mut lhs: T, rhs: &'a T) -> T {
    lhs.union_with(rhs);
    lhs
}
fn pair_difference<'a, T: DifferenceWith<&'a T>>(mut lhs: T, rhs: &'a T) -> T {
    lhs.difference_with(rhs);
    lhs
}
fn pair_symmetric_difference<'a, T: SymmetricDifferenceWith<&'a T>>(mut lhs: T, rhs: &'a T) -> T {
    lhs.symmetric_difference_with(rhs);
    lhs
}

impl<'r> IntersectionWith<&'r Map32> for Map32 {
    fn intersection_with(&mut self, that: &'r Map32) {
        let rms = {
            let mut rms = Vec::with_capacity(self.map16s.len());
            for (key, block) in &mut self.map16s {
                if that.map16s.contains_key(key) {
                    block.intersection_with(&that.map16s[key]);
                    if block.count1() != 0 {
                        block.optimize();
                    } else {
                        rms.push(*key);
                    }
                } else {
                    rms.push(*key);
                }
            }
            rms
        };
        for rm in &rms {
            let removed = self.map16s.remove(rm);
            debug_assert!(removed.is_some());
        }
    }
}

impl<'r> UnionWith<&'r Map32> for Map32 {
    fn union_with(&mut self, that: &'r Map32) {
        for (&key, block) in &that.map16s {
            let rhs = (**block).clone();
            let thunk = if self.map16s.contains_key(&key) {
                let lhs = (*self.map16s[&key]).clone();
                lazy!(pair_union(lhs, &rhs))
            } else {
                eval!(rhs)
            };
            self.map16s.insert(key, thunk);
        }
    }
}

impl<'r> DifferenceWith<&'r Map32> for Map32 {
    fn difference_with(&mut self, that: &'r Map32) {
        for (&key, block) in &mut self.map16s {
            if !that.map16s.contains_key(&key) {
                continue;
            }
            let lhs = (**block).clone();
            let rhs = (*that.map16s[&key]).clone();
            *block = lazy!(pair_difference(lhs, &rhs));
        }
    }
}

impl<'r> SymmetricDifferenceWith<&'r Map32> for Map32 {
    fn symmetric_difference_with(&mut self, that: &'r Map32) {
        for (&key, block) in &that.map16s {
            let rhs = (**block).clone();
            let thunk = if self.map16s.contains_key(&key) {
                let lhs = (*self.map16s[&key]).clone();
                lazy!(pair_symmetric_difference(lhs, &rhs))
            } else {
                eval!(rhs)
            };
            self.map16s.insert(key, thunk);
        }
    }
}

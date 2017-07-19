use std::fmt::{self, Debug, Formatter};
use std::collections::BTreeMap;
use karabiner::thunk::Thunk;

use {Map16, Rank, Select0, Select1};
use prim::{Merge, Split};
use pair::*;
use block;

type Lazy<T> = Thunk<'static, T>;

/// Map of Map16.
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
    pub fn count_ones(&self) -> u64 {
        self.map16s
            .values()
            .map(|b| u64::from(b.count_ones()))
            .sum()
    }

    pub fn count_zeros(&self) -> u64 {
        (1 << 32) - self.count_ones()
    }

    pub fn mem_size(&self) -> u64 {
        let mut sum = 0;
        for size in self.map16s.values().map(|b| b.mem_size() as u64) {
            sum += size;
        }
        sum
    }

    /// Optimize innternal data representaions.
    pub fn optimize(&mut self) {
        let mut rs = Vec::new();
        for (k, b) in &mut self.map16s {
            b.optimize();
            if b.count_ones() == 0 {
                rs.push(*k)
            }
        }
        for k in rs {
            self.map16s.remove(&k);
        }
    }
}

impl Map32 {
    pub fn new() -> Self {
        Map32 {
            map16s: BTreeMap::new(),
        }
    }

    pub fn stats<'a>(&'a self) -> impl Iterator<Item = block::Stats> + 'a {
        self.map16s.values().map(|v16| v16.stats())
    }

    /// Clear contents.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compacts_bits::Map32;
    ///
    /// let mut bits = Map32::new();
    /// bits.insert(0);
    /// assert!(bits.count_ones() == 1);
    /// bits.clear();
    /// assert!(bits.count_ones() == 0);
    /// ```
    pub fn clear(&mut self) {
        self.map16s.clear();
    }

    /// Return `true` if the value exists.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compacts_bits::Map32;
    ///
    /// let mut bits = Map32::new();
    /// assert_eq!(bits.count_zeros(), 1 << 32);
    /// bits.insert(1);
    /// assert!(!bits.contains(0));
    /// assert!(bits.contains(1));
    /// assert!(!bits.contains(2));
    /// assert_eq!(bits.count_ones(), 1);
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
    /// use compacts_bits::Map32;
    /// let mut bits = Map32::new();
    /// assert!(bits.insert(3));
    /// assert!(!bits.insert(3));
    /// assert!(bits.contains(3));
    /// assert_eq!(bits.count_ones(), 1);
    /// ```
    pub fn insert(&mut self, x: u32) -> bool {
        let (key, bit) = x.split();
        let mut b = self.map16s
            .entry(key)
            .or_insert_with(|| eval!(Map16::new()));
        b.insert(bit)
    }

    /// Return `true` if the value exists and removed successfuly.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compacts_bits::Map32;
    /// let mut bits = Map32::new();
    /// assert!(bits.insert(3));
    /// assert!(bits.remove(3));
    /// assert!(!bits.contains(3));
    /// assert_eq!(bits.count_ones(), 0);
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
}

impl ::std::ops::Index<u32> for Map32 {
    type Output = bool;
    fn index(&self, i: u32) -> &Self::Output {
        if self.contains(i) {
            super::TRUE
        } else {
            super::FALSE
        }
    }
}

impl Rank<u32> for Map32 {
    type Count = u64;

    fn rank1(&self, i: u32) -> Self::Count {
        let (hi, lo) = i.split();
        let mut rank = 0;
        for (&key, block) in &self.map16s {
            if key > hi {
                break;
            } else if key == hi {
                rank += Self::Count::from(block.rank1(lo));
                break;
            } else {
                rank += Self::Count::from(block.count_ones());
            }
        }
        rank
    }

    fn rank0(&self, i: u32) -> Self::Count {
        let rank1 = self.rank1(i);
        i as Self::Count + 1 - rank1
    }
}

impl Select1<u32> for Map32 {
    type Index = u32;

    fn select1(&self, c: u32) -> Option<Self::Index> {
        if self.count_ones() <= c as u64 {
            return None;
        }
        let mut rem = c;
        for (&key, b) in &self.map16s {
            let w = b.count_ones();
            if rem >= w {
                rem -= w;
            } else {
                let s = b.select1(rem as u16).unwrap() as u32;
                let k = (key as u32) << 16;
                return Some(k + s);
            }
        }
        None
    }
}

impl Select0<u32> for Map32 {
    type Index = u32;

    fn select0(&self, c: u32) -> Option<Self::Index> {
        use Rank;
        if self.count_zeros() <= c as u64 {
            return None;
        }

        let fun = |i| {
            let rank0 = self.rank0(i as u32);
            rank0 > c as u64
        };
        let pos = search!(0u64, 1 << 32, fun);
        if pos < (1 << 32) {
            Some(pos as u32)
        } else {
            None
        }
    }
}

macro_rules! impl_Pairwise {
    ( $( ( $op:ident, $fn:ident, $fn_with:ident ) ),* ) => ($(
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
                    if block.count_ones() != 0 {
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

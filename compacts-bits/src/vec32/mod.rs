mod pairwise;
// mod similarity;

use std::fmt::{self, Debug, Formatter};
use std::collections::BTreeMap;
use karabiner::thunk::Thunk;
use {Vec16, Split, Merge};

type Lazy<T> = Thunk<'static, T>;

/// Map of Vec16(internal).
#[derive(Default)]
pub struct Vec32 {
    vec16s: BTreeMap<u16, Lazy<Vec16>>,
}

impl Debug for Vec32 {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let b = self.count_vec16s();
        let w = self.count_ones();
        write!(f, "Vec32{{ vec16s={:?} weight={:?} }}", b, w)
    }
}
impl Clone for Vec32 {
    fn clone(&self) -> Self {
        let mut vec = Vec32::new();
        for (&k, t) in &self.vec16s {
            let c = (**t).clone();
            vec.vec16s.insert(k, eval!(c));
        }
        vec
    }
}

impl Vec32 {
    pub fn count_ones(&self) -> u64 {
        self.vec16s
            .values()
            .map(|b| u64::from(b.count_ones()))
            .sum()
    }

    pub fn count_zeros(&self) -> u64 {
        (1 << 32) - self.count_ones()
    }

    pub fn mem_size(&self) -> u64 {
        let mut sum = 0;
        for size in self.vec16s.values().map(|b| b.mem_size() as u64) {
            sum += size;
        }
        sum
    }

    fn count_vec16s(&self) -> usize {
        self.vec16s.len()
    }

    /// Optimize innternal data representaions.
    pub fn optimize(&mut self) {
        let mut rs = Vec::new();
        for (k, b) in self.vec16s.iter_mut() {
            b.optimize();
            if b.count_ones() == 0 {
                rs.push(*k)
            }
        }
        for k in rs {
            self.vec16s.remove(&k);
        }
    }
}

impl Vec32 {
    pub fn new() -> Self {
        Vec32 { vec16s: BTreeMap::new() }
    }

    /// Clear contents.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compacts_bits::Vec32;
    ///
    /// let mut bits = Vec32::new();
    /// bits.insert(0);
    /// assert!(bits.count_ones() == 1);
    /// bits.clear();
    /// assert!(bits.count_ones() == 0);
    /// ```
    pub fn clear(&mut self) {
        self.vec16s.clear();
    }

    /// Return `true` if the value exists.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compacts_bits::Vec32;
    ///
    /// let mut bits = Vec32::new();
    /// assert_eq!(bits.count_zeros(), 1 << 32);
    /// bits.insert(1);
    /// assert!(!bits.contains(0));
    /// assert!(bits.contains(1));
    /// assert!(!bits.contains(2));
    /// assert_eq!(bits.count_ones(), 1);
    /// ```
    pub fn contains(&self, x: u32) -> bool {
        let (key, bit) = x.split();
        self.vec16s.get(&key).map_or(false, |b| b.contains(bit))
    }

    /// Return `true` if the value doesn't exists and inserted successfuly.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compacts_bits::Vec32;
    /// let mut bits = Vec32::new();
    /// assert!(bits.insert(3));
    /// assert!(!bits.insert(3));
    /// assert!(bits.contains(3));
    /// assert_eq!(bits.count_ones(), 1);
    /// ```
    pub fn insert(&mut self, x: u32) -> bool {
        let (key, bit) = x.split();
        let mut b = self.vec16s
            .entry(key)
            .or_insert_with(|| eval!(Vec16::new()));
        b.insert(bit)
    }

    /// Return `true` if the value exists and removed successfuly.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compacts_bits::Vec32;
    /// let mut bits = Vec32::new();
    /// assert!(bits.insert(3));
    /// assert!(bits.remove(3));
    /// assert!(!bits.contains(3));
    /// assert_eq!(bits.count_ones(), 0);
    /// ```
    pub fn remove(&mut self, x: u32) -> bool {
        let (key, bit) = x.split();
        if let Some(b) = self.vec16s.get_mut(&key) {
            b.remove(bit)
        } else {
            false
        }
    }

    pub fn iter<'r>(&'r self) -> impl Iterator<Item = u32> + 'r {
        self.vec16s.iter().flat_map(|(&key, block)| {
            block
                .iter()
                .map(move |val| <u32 as Merge>::merge((key, val)))
        })
    }
}

impl ::std::ops::Index<u32> for Vec32 {
    type Output = bool;
    fn index(&self, i: u32) -> &Self::Output {
        if self.contains(i) {
            super::TRUE
        } else {
            super::FALSE
        }
    }
}

impl ::Rank<u32> for Vec32 {
    type Weight = u64;

    const SIZE: Self::Weight = 1 << 32;

    fn rank1(&self, i: u32) -> Self::Weight {
        let (hi, lo) = i.split();
        let mut rank = 0;
        for (&key, block) in &self.vec16s {
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

    fn rank0(&self, i: u32) -> Self::Weight {
        i as Self::Weight + 1 - self.rank1(i)
    }
}

impl ::Select1<u32> for Vec32 {
    fn select1(&self, c: u32) -> Option<u32> {
        if self.count_ones() <= c as u64 {
            return None;
        }
        let mut rem = c;
        for (&key, b) in &self.vec16s {
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

impl ::Select0<u32> for Vec32 {
    fn select0(&self, c: u32) -> Option<u32> {
        if self.count_zeros() <= c as u64 {
            return None;
        }
        let mut rem = c;
        for (&key, b) in &self.vec16s {
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

#[derive(Clone, Debug)]
enum BlockKind {
    Seq16,
    Seq64,
    Rle16,
}

#[derive(Clone, Debug)]
pub struct Stats {
    kind: BlockKind,
    ones: u64,
    size: u64,
}

#[derive(Clone, Debug)]
pub struct Summary {
    seq16_count: usize,
    seq16_ones: u64,
    seq16_size: u64,

    seq64_count: usize,
    seq64_ones: u64,
    seq64_size: u64,

    rle16_count: usize,
    rle16_ones: u64,
    rle16_size: u64,
}

impl Vec32 {
    pub fn stats<'a>(&'a self) -> impl Iterator<Item = Stats> + 'a {
        self.vec16s.values().map(|v16| match **v16 {
            super::Vec16::Seq16(ref b) => Stats {
                kind: BlockKind::Seq16,
                ones: b.count_ones() as u64,
                size: b.mem_size() as u64,
            },
            super::Vec16::Seq64(ref b) => Stats {
                kind: BlockKind::Seq64,
                ones: b.count_ones() as u64,
                size: b.mem_size() as u64,
            },
            super::Vec16::Rle16(ref b) => Stats {
                kind: BlockKind::Rle16,
                ones: b.count_ones() as u64,
                size: b.mem_size() as u64,
            },
        })
    }
}

impl ::std::iter::Sum<Stats> for Summary {
    fn sum<I>(iter: I) -> Summary
    where
        I: Iterator<Item = Stats>,
    {
        let mut sum = Summary {
            seq16_count: 0,
            seq16_ones: 0,
            seq16_size: 0,

            seq64_count: 0,
            seq64_ones: 0,
            seq64_size: 0,

            rle16_count: 0,
            rle16_ones: 0,
            rle16_size: 0,
        };
        for stat in iter {
            match stat.kind {
                BlockKind::Seq16 => {
                    sum.seq16_count += 1;
                    sum.seq16_ones += stat.ones;
                    sum.seq16_size += stat.size;
                }
                BlockKind::Seq64 => {
                    sum.seq64_count += 1;
                    sum.seq64_ones += stat.ones;
                    sum.seq64_size += stat.size;
                }
                BlockKind::Rle16 => {
                    sum.rle16_count += 1;
                    sum.rle16_ones += stat.ones;
                    sum.rle16_size += stat.size;
                }
            }
        }
        sum
    }
}

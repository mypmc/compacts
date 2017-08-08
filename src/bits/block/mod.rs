mod seq16;
mod seq64;
mod rle16;
#[cfg(test)]
mod tests;

use std::iter::ExactSizeIterator;
use std::ops::RangeInclusive;

use dict::{Rank, Select0, Select1};
use bits::pair::*;

use self::seq16::Seq16Iter;
use self::seq64::Seq64Iter;
use self::rle16::Rle16Iter;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Seq<T> {
    pub(crate) weight: u32,
    pub(crate) vector: Vec<T>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Rle<T> {
    pub(crate) weight: u32,
    pub(crate) ranges: Vec<RangeInclusive<T>>,
}

pub(crate) type Seq16 = Seq<u16>;
pub(crate) type Seq64 = Seq<u64>;
pub(crate) type Rle16 = Rle<u16>;

#[derive(Clone, Debug)]
pub(crate) enum Block {
    Seq16(Seq16),
    Seq64(Seq64),
    Rle16(Rle16),
}

pub enum Iter<'a> {
    Seq16(Seq16Iter<'a>),
    Seq64(Seq64Iter<'a>),
    Rle16(Rle16Iter<'a>),
}

#[derive(Clone, Debug)]
pub(crate) enum Kind {
    Seq16,
    Seq64,
    Rle16,
}

/// Stats of block.
/// 'ones' is a count of non-zero bits.
/// 'size' is an approximate size in bytes.
#[derive(Clone, Debug)]
pub struct Stats {
    pub(crate) kind: Kind,
    pub ones: u64,
    pub size: u64,
}

impl Default for Block {
    fn default() -> Self {
        Block::Seq64(Seq64::new())
    }
}

impl Block {
    pub const CAPACITY: usize = 1 << 16;

    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }

    pub fn iter(&self) -> Iter {
        match *self {
            Block::Seq16(ref seq) => Iter::from(seq.iter()),
            Block::Seq64(ref seq) => Iter::from(seq.iter()),
            Block::Rle16(ref rle) => Iter::from(rle.iter()),
        }
    }

    pub fn stats(&self) -> Stats {
        match *self {
            Block::Seq16(ref b) => Stats {
                kind: Kind::Seq16,
                ones: b.count_ones() as u64,
                size: b.mem_size() as u64,
            },
            Block::Seq64(ref b) => Stats {
                kind: Kind::Seq64,
                ones: b.count_ones() as u64,
                size: b.mem_size() as u64,
            },
            Block::Rle16(ref b) => Stats {
                kind: Kind::Rle16,
                ones: b.count_ones() as u64,
                size: b.mem_size() as u64,
            },
        }
    }

    pub fn contains(&self, bit: u16) -> bool {
        match *self {
            Block::Seq16(ref seq) => seq.contains(bit),
            Block::Seq64(ref seq) => seq.contains(bit),
            Block::Rle16(ref rle) => rle.contains(bit),
        }
    }

    pub fn insert(&mut self, bit: u16) -> bool {
        match *self {
            Block::Seq16(ref mut seq) => seq.insert(bit),
            Block::Seq64(ref mut seq) => seq.insert(bit),
            Block::Rle16(ref mut rle) => rle.insert(bit),
        }
    }

    pub fn remove(&mut self, bit: u16) -> bool {
        match *self {
            Block::Seq16(ref mut seq) => seq.remove(bit),
            Block::Seq64(ref mut seq) => seq.remove(bit),
            Block::Rle16(ref mut rle) => rle.remove(bit),
        }
    }

    pub fn count_ones(&self) -> u32 {
        match *self {
            Block::Seq16(ref seq) => seq.weight,
            Block::Seq64(ref seq) => seq.weight,
            Block::Rle16(ref rle) => rle.weight,
        }
    }

    pub fn count_zeros(&self) -> u32 {
        match *self {
            Block::Seq16(ref seq) => Self::CAPACITY as u32 - seq.weight,
            Block::Seq64(ref seq) => Self::CAPACITY as u32 - seq.weight,
            Block::Rle16(ref rle) => Self::CAPACITY as u32 - rle.weight,
        }
    }

    pub fn count_rle(&self) -> usize {
        match *self {
            Block::Seq16(ref seq) => {
                let rle = Rle16::from(seq);
                rle.ranges.len()
            }
            Block::Seq64(ref seq) => {
                let rle = Rle16::from(seq);
                rle.ranges.len()
            }
            Block::Rle16(ref rle) => rle.ranges.len(),
        }
    }

    pub fn shrink_to_fit(&mut self) {
        match *self {
            Block::Seq16(ref mut seq) => seq.vector.shrink_to_fit(),
            Block::Seq64(ref mut seq) => seq.vector.shrink_to_fit(),
            Block::Rle16(ref mut rle) => rle.ranges.shrink_to_fit(),
        }
    }

    pub fn mem_size(&self) -> usize {
        match *self {
            Block::Seq16(ref seq) => seq.mem_size(),
            Block::Seq64(ref seq) => seq.mem_size(),
            Block::Rle16(ref rle) => rle.mem_size(),
        }
    }

    pub fn as_seq64(&mut self) {
        *self = match *self {
            Block::Seq16(ref seq) => Block::Seq64(Seq64::from(seq)),
            Block::Rle16(ref rle) => Block::Seq64(Seq64::from(rle)),
            _ => unreachable!(),
        }
    }

    /// Convert to more efficient block representaions.
    pub fn optimize(&mut self) {
        const SEQ16: usize = 4096; // 4096 * 16 == 65536
        const SEQ64: usize = 1024; // 1024 * 64 == 65536

        let mem_size = self.mem_size();

        let new_block = match *self {
            Block::Seq16(ref seq) => {
                let mem_in_seq16 = mem_size;
                let mem_in_seq64 = Seq64::size(SEQ64);
                let rle = Rle16::from(seq);
                let mem_in_rle16 = Rle16::size(rle.count_rle());

                if mem_in_rle16 <= ::std::cmp::min(mem_in_seq64, mem_in_seq16) {
                    Some(Block::Rle16(rle))
                } else if self.count_ones() as usize <= SEQ16 {
                    None
                } else {
                    Some(Block::Seq64(Seq64::from(seq)))
                }
            }

            Block::Seq64(ref seq) => {
                let mem_in_seq16 = Seq16::size(seq.count_ones() as usize);
                let mem_in_seq64 = mem_size;
                let rle = Rle16::from(seq);
                let mem_in_rle16 = Rle16::size(rle.count_rle());

                if mem_in_rle16 <= ::std::cmp::min(mem_in_seq64, mem_in_seq16) {
                    Some(Block::Rle16(rle))
                } else if seq.weight as usize <= SEQ16 {
                    Some(Block::Seq16(Seq16::from(seq)))
                } else {
                    None
                }
            }

            Block::Rle16(ref rle) => {
                let mem_in_seq16 = Seq16::size(rle.count_ones() as usize);
                let mem_in_seq64 = Seq64::size(SEQ64);
                let mem_in_rle16 = mem_size;

                if mem_in_rle16 <= ::std::cmp::min(mem_in_seq64, mem_in_seq16) {
                    None
                } else if rle.weight as usize <= SEQ16 {
                    Some(Block::Seq16(Seq16::from(rle)))
                } else {
                    Some(Block::Seq64(Seq64::from(rle)))
                }
            }
        };
        if let Some(block) = new_block {
            *self = block;
        }
    }
}

impl Rank<u16> for Block {
    type Count = u32;

    fn rank1(&self, i: u16) -> Self::Count {
        // assert!(i > 0);
        // if i as usize >= Self::CAPACITY {
        //     return self.count_ones();
        // }

        match *self {
            Block::Seq16(ref seq) => {
                let vec = &seq.vector;
                let fun = |p| vec.get(p).map_or(false, |&v| v >= i);
                search!(0, vec.len(), fun) as Self::Count
            }

            Block::Seq64(ref seq) => {
                let q = (i / 64) as usize;
                let r = (i % 64) as u32;
                let vec = &seq.vector;
                vec.iter().take(q).fold(0, |acc, w| acc + w.count_ones()) + (if r == 0 {
                    0
                } else {
                    vec.get(q).map_or(0, |w| w.rank1(r))
                })
            }

            Block::Rle16(ref rle) => match rle.search(&i) {
                Err(n) => if n >= rle.ranges.len() {
                    rle.weight
                } else {
                    rle.ranges
                        .iter()
                        .map(|r| (r.end - r.start) as u32 + 1)
                        .take(n)
                        .sum::<u32>()
                },
                Ok(n) => {
                    let r = rle.ranges
                        .iter()
                        .map(|r| (r.end - r.start) as u32 + 1)
                        .take(n)
                        .sum::<u32>();
                    (i as u32 - (rle.ranges[n].start as u32)) + r
                }
            },
        }
    }

    fn rank0(&self, i: u16) -> Self::Count {
        // assert!(i > 0);
        i as Self::Count - self.rank1(i)
    }
}

impl Select1<u16> for Block {
    type Index = u16;

    fn select1(&self, c: u16) -> Option<Self::Index> {
        if c as u32 >= self.count_ones() {
            return None;
        }
        match *self {
            Block::Seq16(ref seq) => seq.vector.get(c as usize).cloned(),

            Block::Seq64(ref seq) => {
                let mut remain = c as u32;
                for (i, bit) in seq.vector.iter().enumerate().filter(|&(_, v)| *v != 0) {
                    let ones = bit.count_ones();
                    if remain < ones {
                        let width = 64;
                        let select = bit.select1(remain).unwrap_or(0);
                        return Some((width * i) as u16 + select as u16);
                    }
                    remain -= ones;
                }
                None
            }

            Block::Rle16(ref rle) => {
                let mut curr = 0;
                for range in &rle.ranges {
                    let next = curr + (range.end - range.start + 1);
                    if next > c {
                        return Some(range.start - curr + c);
                    }
                    curr = next;
                }
                None
            }
        }
    }
}

impl Select0<u16> for Block {
    type Index = u16;

    fn select0(&self, c: u16) -> Option<Self::Index> {
        if c as u32 >= self.count_zeros() {
            return None;
        }
        let c32 = c as u32;
        match *self {
            Block::Seq16(_) | Block::Rle16(_) => {
                let cap = Self::CAPACITY as u32;
                let fun = |i| self.rank0(i as u16) > c32;
                let pos = search!(0, cap, fun);
                if pos < Self::CAPACITY as u32 {
                    Some(pos as u16 - 1)
                } else {
                    None
                }
            }

            Block::Seq64(ref seq) => {
                let mut remain = c32;
                for (i, bit) in seq.vector.iter().enumerate() {
                    let zeros = bit.count_zeros();
                    if remain < zeros {
                        let width = 64;
                        let select = bit.select0(remain).unwrap_or(0);
                        return Some((width * i) as u16 + select as u16);
                    }
                    remain -= zeros;
                }
                None
            }
        }
    }
}

impl ::std::iter::FromIterator<u16> for Block {
    fn from_iter<I>(iterable: I) -> Self
    where
        I: ::std::iter::IntoIterator<Item = u16>,
    {
        let iter = iterable.into_iter();
        let mut block = Block::new();
        let ones = extend_by_u16!(&mut block, iter);
        debug_assert_eq!(ones, block.count_ones());
        block
    }
}
impl<'a> ::std::iter::FromIterator<&'a u16> for Block {
    fn from_iter<I>(iterable: I) -> Self
    where
        I: ::std::iter::IntoIterator<Item = &'a u16>,
    {
        let iter = iterable.into_iter();
        iter.cloned().collect::<Self>()
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = u16;
    fn next(&mut self) -> Option<Self::Item> {
        match *self {
            Iter::Seq16(ref mut it) => it.next(),
            Iter::Seq64(ref mut it) => it.next(),
            Iter::Rle16(ref mut it) => it.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match *self {
            Iter::Seq16(ref it) => it.size_hint(),
            Iter::Seq64(ref it) => it.size_hint(),
            Iter::Rle16(ref it) => it.size_hint(),
        }
    }
}

impl<'a> ExactSizeIterator for Iter<'a> {
    fn len(&self) -> usize {
        match *self {
            Iter::Seq16(ref it) => it.len(),
            Iter::Seq64(ref it) => it.len(),
            Iter::Rle16(ref it) => it.len(),
        }
    }
}

impl<'a> From<Seq16Iter<'a>> for Iter<'a> {
    fn from(iter: Seq16Iter<'a>) -> Self {
        Iter::Seq16(iter)
    }
}

impl<'a> From<Seq64Iter<'a>> for Iter<'a> {
    fn from(iter: Seq64Iter<'a>) -> Self {
        Iter::Seq64(iter)
    }
}

impl<'a> From<Rle16Iter<'a>> for Iter<'a> {
    fn from(iter: Rle16Iter<'a>) -> Self {
        Iter::Rle16(iter)
    }
}

macro_rules! impl_Pairwise {
    ( $( ( $op:ident, $fn:ident, $fn_with:ident ) ),* ) => ($(
        impl<'a> $op<&'a Block> for Block {
            type Output = Block;
            fn $fn(self, that: &Block) -> Self::Output {
                let mut this = self;
                this.$fn_with(that);
                this
            }
        }

        impl<'a, 'b> $op<&'b Block> for &'a Block {
            type Output = Block;
            fn $fn(self, that: &Block) -> Self::Output {
                match (self, that) {
                    (this @ &Block::Seq16(..), that @ &Block::Seq16(..)) => {
                        ::bits::pair::$fn(this.iter(), that.iter()).collect()
                    }

                    (&Block::Rle16(ref b1), &Block::Rle16(ref b2)) => Block::Rle16(b1.$fn(b2)),

                    (this, that) => {
                        let mut this = this.clone();
                        this.$fn_with(that);
                        this
                    }
                }
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

impl<'a> IntersectionWith<&'a Block> for Block {
    fn intersection_with(&mut self, target: &Block) {
        match (self, target) {
            (&mut Block::Seq16(ref mut b1), &Block::Seq16(ref b2)) => b1.intersection_with(b2),

            (&mut Block::Seq16(ref mut b1), &Block::Seq64(ref b2)) => {
                let weight = {
                    let mut new = 0;
                    for i in 0..b1.vector.len() {
                        if b2.contains(b1.vector[i]) {
                            b1.vector[new] = b1.vector[i];
                            new += 1;
                        }
                    }
                    new
                };
                b1.vector.truncate(weight);
                b1.weight = weight as u32;
            }

            (&mut Block::Seq64(ref mut b1), &Block::Seq64(ref b2)) => b1.intersection_with(b2),

            (&mut Block::Seq64(ref mut b1), &Block::Seq16(ref b2)) => {
                let new = Seq64::from(b2);
                b1.intersection_with(&new);
            }

            (&mut Block::Seq64(ref mut b1), &Block::Rle16(ref b2)) => {
                let new = Seq64::from(b2);
                b1.intersection_with(&new);
            }

            (&mut Block::Rle16(ref mut b1), &Block::Rle16(ref b2)) => b1.intersection_with(b2),

            (this, that) => {
                this.as_seq64();
                this.intersection_with(that);
            }
        }
    }
}

impl<'a> UnionWith<&'a Block> for Block {
    fn union_with(&mut self, target: &Block) {
        match (self, target) {
            (&mut Block::Seq16(ref mut b1), &Block::Seq16(ref b2)) => b1.union_with(b2),

            (&mut Block::Seq64(ref mut b1), &Block::Seq64(ref b2)) => b1.union_with(b2),

            (&mut Block::Seq64(ref mut b1), &Block::Seq16(ref b2)) => for &bit in &b2.vector {
                b1.insert(bit);
            },

            (&mut Block::Seq64(ref mut b1), &Block::Rle16(ref b2)) => for range in &b2.ranges {
                for bit in range.start...range.end {
                    b1.insert(bit);
                }
            },

            (&mut Block::Rle16(ref mut b1), &Block::Rle16(ref b2)) => b1.union_with(b2),

            (this, that) => {
                this.as_seq64();
                this.union_with(that);
            }
        }
    }
}

impl<'a> DifferenceWith<&'a Block> for Block {
    fn difference_with(&mut self, target: &Block) {
        match (self, target) {
            (&mut Block::Seq16(ref mut b1), &Block::Seq16(ref b2)) => b1.difference_with(b2),

            (&mut Block::Seq64(ref mut b1), &Block::Seq64(ref b2)) => b1.difference_with(b2),

            (&mut Block::Seq64(ref mut b1), &Block::Seq16(ref b2)) => for &bit in &b2.vector {
                b1.remove(bit);
            },

            (&mut Block::Seq64(ref mut b1), &Block::Rle16(ref b2)) => for range in &b2.ranges {
                for bit in range.start...range.end {
                    b1.remove(bit);
                }
            },

            (&mut Block::Rle16(ref mut b1), &Block::Rle16(ref b2)) => b1.difference_with(b2),

            (this, that) => {
                this.as_seq64();
                this.difference_with(that);
            }
        }
    }
}

impl<'a> SymmetricDifferenceWith<&'a Block> for Block {
    fn symmetric_difference_with(&mut self, target: &Block) {
        match (self, target) {
            (&mut Block::Seq16(ref mut b1), &Block::Seq16(ref b2)) => {
                b1.symmetric_difference_with(b2)
            }

            (&mut Block::Seq64(ref mut b1), &Block::Seq64(ref b2)) => {
                b1.symmetric_difference_with(b2)
            }

            (&mut Block::Seq64(ref mut b1), &Block::Seq16(ref b2)) => for &bit in &b2.vector {
                if b1.contains(bit) {
                    b1.remove(bit);
                } else {
                    b1.insert(bit);
                }
            },

            (&mut Block::Seq64(ref mut b1), &Block::Rle16(ref b2)) => for range in &b2.ranges {
                for bit in range.start...range.end {
                    if b1.contains(bit) {
                        b1.remove(bit);
                    } else {
                        b1.insert(bit);
                    }
                }
            },

            (&mut Block::Rle16(ref mut b1), &Block::Rle16(ref b2)) => {
                b1.symmetric_difference_with(b2)
            }

            (this, that) => {
                this.as_seq64();
                this.symmetric_difference_with(that);
            }
        }
    }
}

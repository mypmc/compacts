macro_rules! delegate {
    ( $this:ident, $method:ident $(, $args: expr )* ) => {{
        $this.block.$method( $( $args ),* )
    }};
}

// #[cfg(test)] mod tests;

use std::fmt;
use {Rank, Select0, Select1};
use pair::*;
use block::{self, Block};

#[derive(Clone)]
pub struct Map16 {
    block: Block,
}

impl fmt::Debug for Map16 {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{:?}", self.block)
    }
}

impl Default for Map16 {
    fn default() -> Self {
        Map16 {
            block: Block::default(),
        }
    }
}

impl Map16 {
    pub const CAPACITY: u32 = Block::CAPACITY as u32;

    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn clear(&mut self) {
        delegate!(self, clear)
    }

    /// May convert to more efficient block representaions.
    /// This may consume time and resource. So, don't call too much.
    #[inline]
    pub fn optimize(&mut self) {
        delegate!(self, optimize)
    }

    #[inline]
    pub fn mem_size(&self) -> usize {
        delegate!(self, mem_size)
    }

    #[inline]
    pub fn count_ones(&self) -> u32 {
        delegate!(self, count_ones)
    }

    #[inline]
    pub fn count_zeros(&self) -> u32 {
        delegate!(self, count_zeros)
    }

    #[inline]
    pub fn contains(&self, bit: u16) -> bool {
        delegate!(self, contains, bit)
    }

    #[inline]
    pub fn insert(&mut self, bit: u16) -> bool {
        delegate!(self, insert, bit)
    }

    #[inline]
    pub fn remove(&mut self, bit: u16) -> bool {
        delegate!(self, remove, bit)
    }

    #[inline]
    pub fn iter(&self) -> block::Iter {
        delegate!(self, iter)
    }

    #[inline]
    pub fn stats(&self) -> block::Stats {
        delegate!(self, stats)
    }
}

impl ::std::ops::Index<u16> for Map16 {
    type Output = bool;
    fn index(&self, i: u16) -> &Self::Output {
        if self.contains(i) {
            ::TRUE
        } else {
            ::FALSE
        }
    }
}

impl<'a> ::std::iter::IntoIterator for &'a Map16 {
    type Item = <block::Iter<'a> as Iterator>::Item;
    type IntoIter = block::Iter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl ::std::iter::Extend<u16> for Map16 {
    fn extend<I>(&mut self, iterable: I)
    where
        I: ::std::iter::IntoIterator<Item = u16>,
    {
        extend_by_u16!(self, iterable);
    }
}

impl ::std::iter::FromIterator<u16> for Map16 {
    fn from_iter<I>(iterable: I) -> Self
    where
        I: ::std::iter::IntoIterator<Item = u16>,
    {
        let iter = iterable.into_iter();
        let mut block = Map16::new();
        let ones = extend_by_u16!(&mut block, iter);
        debug_assert_eq!(ones, block.count_ones());
        block
    }
}
impl<'a> ::std::iter::FromIterator<&'a u16> for Map16 {
    fn from_iter<I>(iterable: I) -> Self
    where
        I: ::std::iter::IntoIterator<Item = &'a u16>,
    {
        let iter = iterable.into_iter();
        iter.cloned().collect::<Self>()
    }
}

impl ::std::iter::FromIterator<bool> for Map16 {
    fn from_iter<I>(iterable: I) -> Self
    where
        I: ::std::iter::IntoIterator<Item = bool>,
    {
        let iter = iterable.into_iter();
        iter.take(Self::CAPACITY as usize)
            .enumerate()
            .filter_map(|(i, p)| if p { Some(i as u16) } else { None })
            .collect::<Self>()
    }
}
impl<'a> ::std::iter::FromIterator<&'a bool> for Map16 {
    fn from_iter<I>(iterable: I) -> Self
    where
        I: ::std::iter::IntoIterator<Item = &'a bool>,
    {
        let iter = iterable.into_iter();
        iter.cloned().collect::<Self>()
    }
}

impl Rank<u16> for super::Map16 {
    type Count = u32;
    fn rank1(&self, i: u16) -> Self::Count {
        delegate!(self, rank1, i)
    }
    fn rank0(&self, i: u16) -> Self::Count {
        delegate!(self, rank0, i)
    }
}

impl Select1<u16> for super::Map16 {
    type Index = u16;
    fn select1(&self, c: u16) -> Option<Self::Index> {
        delegate!(self, select1, c)
    }
}

impl Select0<u16> for super::Map16 {
    type Index = u16;
    fn select0(&self, c: u16) -> Option<Self::Index> {
        delegate!(self, select0, c)
    }
}

macro_rules! impl_Pairwise {
    ( $( ( $op:ident, $fn:ident ) ),* ) => ($(
        impl<'a> $op<&'a Map16> for Map16 {
            type Output = Map16;
            fn $fn(self, that: &Map16) -> Self::Output {
                Map16 { block: self.block.$fn(&that.block) }
            }
        }
        impl<'a, 'b> $op<&'b Map16> for &'a Map16 {
            type Output = Map16;
            fn $fn(self, that: &Map16) -> Self::Output {
                Map16 { block: (&self.block).$fn(&that.block) }
            }
        }
    )*)
}

impl_Pairwise!(
    (Intersection, intersection),
    (Union, union),
    (Difference, difference),
    (SymmetricDifference, symmetric_difference)
);

macro_rules! impl_PairwiseWith {
    ( $( ( $op:ident, $fn_with:ident ) ),* ) => ($(
        impl<'a> $op<&'a Map16> for Map16 {
            fn $fn_with(&mut self, that: &Map16) {
                self.block.$fn_with(&that.block)
            }
        }
    )*)
}

impl_PairwiseWith!(
    (IntersectionWith, intersection_with),
    (UnionWith, union_with),
    (DifferenceWith, difference_with),
    (SymmetricDifferenceWith, symmetric_difference_with)
);

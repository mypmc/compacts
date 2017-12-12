use std::{iter, slice};
use std::borrow::Cow;
use std::cmp::Ordering;

use super::{bitops, pair, prim, Block};
use self::prim::Merge;
use self::bitops::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry<'a> {
    inner: Cow<'a, Block>,
}
impl<'a> PartialOrd for Entry<'a> {
    fn partial_cmp(&self, that: &Self) -> Option<Ordering> {
        let s1 = self.inner.slot();
        let s2 = that.inner.slot();
        s1.partial_cmp(&s2)
    }
}
impl<'a> Ord for Entry<'a> {
    fn cmp(&self, that: &Self) -> Ordering {
        let s1 = self.inner.slot();
        let s2 = that.inner.slot();
        s1.cmp(&s2)
    }
}

#[derive(Debug)]
pub struct Entries<'a>(iter::Map<slice::Iter<'a, Block>, ToEntry>);

type ToEntry = for<'a> fn(&'a Block) -> Entry<'a>;

impl<'a> Entry<'a> {
    pub fn bits<'r>(&'r self) -> impl Iterator<Item = u32> + 'r {
        let slot = self.inner.slot;
        self.inner
            .repr
            .iter()
            .map(move |half| <u32 as Merge>::merge((slot, half)))
    }

    fn into_bits(self) -> impl Iterator<Item = u32> {
        let slot = self.inner.slot();
        self.inner
            .into_owned()
            .repr
            .into_iter()
            .map(move |half| <u32 as Merge>::merge((slot, half)))
    }
}

type Pair<'a, 'b> = (Option<Entry<'a>>, Option<Entry<'b>>);

macro_rules! defops {
    ( $( ( $tyname:ident, $fnname:ident ) ),* ) => ($(
        pub struct $tyname<'a, L, R>
        where
            L: Iterator<Item = Entry<'a>>,
            R: Iterator<Item = Entry<'a>>,
        {
            pair: pair::$tyname<L, R>,
        }
        impl<'a, L, R> Iterator for $tyname<'a, L, R>
        where
            L: Iterator<Item = Entry<'a>>,
            R: Iterator<Item = Entry<'a>>,
        {
            type Item = Entry<'a>;
            fn next(&mut self) -> Option<Self::Item> {
                self.pair.next().and_then($fnname)
            }
        }

        impl<'a, L, R> $tyname<'a, L, R>
        where
            L: Iterator<Item = Entry<'a>>,
            R: Iterator<Item = Entry<'a>>,
        {
            pub fn bits(self) -> impl Iterator<Item = u32> + 'a
            where
                L: 'a,
                R: 'a,
            {
                self.flat_map(|entry| entry.into_bits())
            }

            pub fn and<T>(self, that: T) -> And<'a, Self, T::IntoIter>
            where
                T: IntoIterator<Item = Entry<'a>>,
            {
                and(self, that)
            }
            pub fn or<T>(self, that: T) -> Or<'a, Self, T::IntoIter>
            where
                T: IntoIterator<Item = Entry<'a>>,
            {
                or(self, that)
            }
            pub fn and_not<T>(self, that: T) -> AndNot<'a, Self, T::IntoIter>
            where
                T: IntoIterator<Item = Entry<'a>>,
            {
                and_not(self, that)
            }
            pub fn xor<T>(self, that: T) -> Xor<'a, Self, T::IntoIter>
            where
                T: IntoIterator<Item = Entry<'a>>,
            {
                xor(self, that)
            }
        }
    )*)
}
defops!((And, _and), (Or, _or), (AndNot, _andnot), (Xor, _xor));

pub fn and<'a, L, R>(lhs: L, rhs: R) -> And<'a, L::IntoIter, R::IntoIter>
where
    L: IntoIterator<Item = Entry<'a>>,
    R: IntoIterator<Item = Entry<'a>>,
{
    And {
        pair: pair::and(lhs, rhs),
    }
}

pub fn or<'a, L, R>(lhs: L, rhs: R) -> Or<'a, L::IntoIter, R::IntoIter>
where
    L: IntoIterator<Item = Entry<'a>>,
    R: IntoIterator<Item = Entry<'a>>,
{
    Or {
        pair: pair::or(lhs, rhs),
    }
}

pub fn and_not<'a, L, R>(lhs: L, rhs: R) -> AndNot<'a, L::IntoIter, R::IntoIter>
where
    L: IntoIterator<Item = Entry<'a>>,
    R: IntoIterator<Item = Entry<'a>>,
{
    AndNot {
        pair: pair::and_not(lhs, rhs),
    }
}

pub fn xor<'a, L, R>(lhs: L, rhs: R) -> Xor<'a, L::IntoIter, R::IntoIter>
where
    L: IntoIterator<Item = Entry<'a>>,
    R: IntoIterator<Item = Entry<'a>>,
{
    Xor {
        pair: pair::xor(lhs, rhs),
    }
}

fn _and<'a, 'b, 'r>(pair: Pair<'a, 'b>) -> Option<Entry<'r>>
where
    'a: 'r,
    'b: 'r,
{
    match pair {
        (Some(mut lhs), Some(rhs)) => {
            lhs.inner.to_mut().repr.bitand_assign(rhs.inner.repr());
            Some(lhs)
        }
        _ => None,
    }
}

fn _or<'a, 'b, 'r>(pair: Pair<'a, 'b>) -> Option<Entry<'r>>
where
    'a: 'r,
    'b: 'r,
{
    match pair {
        (Some(mut lhs), Some(rhs)) => {
            lhs.inner.to_mut().repr.bitor_assign(rhs.inner.repr());
            Some(lhs)
        }
        (Some(lhs), None) => Some(lhs),
        (None, Some(rhs)) => Some(rhs),
        (None, None) => None,
    }
}

fn _andnot<'a, 'b, 'r>(pair: Pair<'a, 'b>) -> Option<Entry<'r>>
where
    'a: 'r,
    'b: 'r,
{
    match pair {
        (Some(mut lhs), Some(rhs)) => {
            lhs.inner.to_mut().repr.bitandnot_assign(rhs.inner.repr());
            Some(lhs)
        }
        (Some(lhs), None) => Some(lhs),
        _ => None,
    }
}

fn _xor<'a, 'b, 'r>(pair: Pair<'a, 'b>) -> Option<Entry<'r>>
where
    'a: 'r,
    'b: 'r,
{
    match pair {
        (Some(mut lhs), Some(rhs)) => {
            lhs.inner.to_mut().repr.bitxor_assign(rhs.inner.repr());
            Some(lhs)
        }
        (Some(lhs), None) => Some(lhs),
        (None, Some(rhs)) => Some(rhs),
        _ => None,
    }
}

impl<'a> Iterator for Entries<'a> {
    type Item = Entry<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}
impl<'a> Entries<'a> {
    pub fn bits(self) -> impl Iterator<Item = u32> + 'a {
        self.flat_map(|entry| entry.into_bits())
    }
}

impl<'a> From<&'a Block> for Entry<'a> {
    fn from(block: &'a Block) -> Entry<'a> {
        let inner = Cow::Borrowed(block);
        Entry { inner }
    }
}

fn to_entry(block: &Block) -> Entry {
    Entry::from(block)
}

impl<'a> IntoIterator for &'a super::Set {
    type Item = Entry<'a>;
    type IntoIter = Entries<'a>;
    fn into_iter(self) -> Self::IntoIter {
        Entries(self.blocks.iter().map(to_entry))
    }
}

impl<'a> iter::FromIterator<Entry<'a>> for super::Set {
    // assume I is sorted by key and all keys are unique.
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Entry<'a>>,
    {
        let mut blocks = Vec::new();
        for e in iter {
            let mut block = e.inner.into_owned();
            block.repr.optimize();
            blocks.push(block);
        }
        super::Set { blocks }
    }
}

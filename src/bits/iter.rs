use std::{iter, slice};
use std::borrow::Cow;
use std::cmp::Ordering;

use super::{merge, pair, Slot};
use super::{BitAndAssign, BitAndNotAssign, BitOrAssign, BitXorAssign};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry<'a> {
    inner: Cow<'a, Slot>,
}

#[derive(Debug)]
#[doc(hidden)]
pub struct Entries<'a>(Mapping<'a, Slot, ToEntry>);

type Mapping<'a, T, F> = iter::Map<slice::Iter<'a, T>, F>;

type ToEntry = for<'x> fn(&'x Slot) -> Entry<'x>;

impl<'a> PartialOrd for Entry<'a> {
    fn partial_cmp(&self, that: &Self) -> Option<Ordering> {
        let s1 = self.inner.key;
        let s2 = that.inner.key;
        s1.partial_cmp(&s2)
    }
}
impl<'a> Ord for Entry<'a> {
    fn cmp(&self, that: &Self) -> Ordering {
        let s1 = self.inner.key;
        let s2 = that.inner.key;
        s1.cmp(&s2)
    }
}

impl<'a> Entry<'a> {
    pub fn bits<'r>(&'r self) -> impl Iterator<Item = u32> + 'r {
        let key = self.inner.key;
        self.inner.bits.iter().map(move |half| merge(key, half))
    }

    fn into_bits(self) -> impl Iterator<Item = u32> {
        let key = self.inner.key;
        self.inner
            .into_owned()
            .bits
            .into_iter()
            .map(move |half| merge(key, half))
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
            lhs.inner.to_mut().bits.bitand_assign(&rhs.inner.bits);
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
            lhs.inner.to_mut().bits.bitor_assign(&rhs.inner.bits);
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
            lhs.inner.to_mut().bits.bitandnot_assign(&rhs.inner.bits);
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
            lhs.inner.to_mut().bits.bitxor_assign(&rhs.inner.bits);
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

impl<'a> From<&'a Slot> for Entry<'a> {
    fn from(slot: &'a Slot) -> Entry<'a> {
        let inner = Cow::Borrowed(slot);
        Entry { inner }
    }
}

fn to_entry(slot: &Slot) -> Entry {
    Entry::from(slot)
}

impl super::Set {
    pub fn entries(&self) -> impl Iterator<Item = Entry> {
        // pub fn entries<'a>(&'a self) -> impl Iterator<Item = Entry<'a>> {
        // pub fn entries(&self) -> Entries {
        self.into_iter()
    }

    pub fn and<'a, T>(&'a self, that: T) -> And<'a, impl Iterator<Item = Entry<'a>>, T::IntoIter>
    where
        T: IntoIterator<Item = Entry<'a>>,
    {
        and(self, that)
    }

    pub fn or<'a, T>(&'a self, that: T) -> Or<'a, impl Iterator<Item = Entry<'a>>, T::IntoIter>
    where
        T: IntoIterator<Item = Entry<'a>>,
    {
        or(self, that)
    }

    pub fn and_not<'a, T>(
        &'a self,
        that: T,
    ) -> AndNot<'a, impl Iterator<Item = Entry<'a>>, T::IntoIter>
    where
        T: IntoIterator<Item = Entry<'a>>,
    {
        and_not(self, that)
    }

    pub fn xor<'a, T>(&'a self, that: T) -> Xor<'a, impl Iterator<Item = Entry<'a>>, T::IntoIter>
    where
        T: IntoIterator<Item = Entry<'a>>,
    {
        xor(self, that)
    }
}

impl<'a> IntoIterator for &'a super::Set {
    type Item = Entry<'a>;
    type IntoIter = Entries<'a>;
    fn into_iter(self) -> Self::IntoIter {
        Entries(self.slots.iter().map(to_entry))
    }
}
impl<'a> iter::FromIterator<Entry<'a>> for super::Set {
    // assume I is sorted by key and all keys are unique.
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Entry<'a>>,
    {
        let mut slots = Vec::new();
        for e in iter {
            let mut slot = e.inner.into_owned();
            slot.bits.optimize();
            slots.push(slot);
        }
        super::Set { slots }
    }
}

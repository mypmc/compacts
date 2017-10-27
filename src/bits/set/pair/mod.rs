mod assign;
mod compare;

use std::borrow::Cow;
use std::cmp::Ordering;
use std::marker::PhantomData;
use bits::{sealed, Merge};
pub(crate) use self::assign::Assign;
pub(crate) use self::compare::Compare;

/// Iterator for bit operations.
pub struct Pair<I, O: sealed::Op>(I, PhantomData<O>);

pub type And<I> = Pair<I, sealed::And>;
pub type Or<I> = Pair<I, sealed::Or>;
pub type AndNot<I> = Pair<I, sealed::AndNot>;
pub type Xor<I> = Pair<I, sealed::Xor>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry<'a> {
    pub(crate) key: u16,
    pub(crate) cow: Cow<'a, super::Repr>,
}

impl<'a> PartialOrd for Entry<'a> {
    fn partial_cmp(&self, that: &Self) -> Option<Ordering> {
        Some(self.key.cmp(&that.key))
    }
}
impl<'a> Ord for Entry<'a> {
    fn cmp(&self, that: &Self) -> Ordering {
        self.key.cmp(&that.key)
    }
}

impl<'a> Entry<'a> {
    pub fn bits(self) -> impl Iterator<Item = u32> + 'a {
        let key = self.key;
        self.cow
            .into_owned()
            .into_iter()
            .map(move |low| <u32 as Merge>::merge((key, low)))
    }
}

impl<'a, I, O> Iterator for Pair<I, O>
where
    I: Iterator<Item = Entry<'a>>,
    O: sealed::Op,
{
    type Item = Entry<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<'a, I, O> Pair<I, O>
where
    I: Iterator<Item = Entry<'a>>,
    O: sealed::Op,
{
    pub fn bits(self) -> impl Iterator<Item = u32> + 'a
    where
        I: 'a,
        O: 'a,
    {
        self.into_iter().flat_map(|e| e.bits())
        // use bits::PopCount;
        // self.into_iter().filter(|e| e.cow.count1() != 0).flat_map(|e| e.bits())
    }

    pub fn and<T>(self, that: T) -> And<impl Iterator<Item = Entry<'a>>>
    where
        T: IntoIterator<Item = Entry<'a>>,
    {
        and(self, that)
    }

    pub fn or<T>(self, that: T) -> Or<impl Iterator<Item = Entry<'a>>>
    where
        T: IntoIterator<Item = Entry<'a>>,
    {
        or(self, that)
    }

    pub fn and_not<T>(self, that: T) -> AndNot<impl Iterator<Item = Entry<'a>>>
    where
        T: IntoIterator<Item = Entry<'a>>,
    {
        and_not(self, that)
    }

    pub fn xor<T>(self, that: T) -> Xor<impl Iterator<Item = Entry<'a>>>
    where
        T: IntoIterator<Item = Entry<'a>>,
    {
        xor(self, that)
    }
}

pub fn and<'a, L, R>(lhs: L, rhs: R) -> And<impl Iterator<Item = Entry<'a>>>
where
    L: IntoIterator<Item = Entry<'a>>,
    R: IntoIterator<Item = Entry<'a>>,
{
    let and = Compare::and(lhs, rhs).filter_map(option_and);
    Pair(and, PhantomData)
}

pub fn or<'a, L, R>(lhs: L, rhs: R) -> Or<impl Iterator<Item = Entry<'a>>>
where
    L: IntoIterator<Item = Entry<'a>>,
    R: IntoIterator<Item = Entry<'a>>,
{
    let or = Compare::or(lhs, rhs).filter_map(option_or);
    Pair(or, PhantomData)
}

pub fn and_not<'a, L, R>(lhs: L, rhs: R) -> AndNot<impl Iterator<Item = Entry<'a>>>
where
    L: IntoIterator<Item = Entry<'a>>,
    R: IntoIterator<Item = Entry<'a>>,
{
    let and_not = Compare::and_not(lhs, rhs).filter_map(option_and_not);
    Pair(and_not, PhantomData)
}

pub fn xor<'a, L, R>(lhs: L, rhs: R) -> Xor<impl Iterator<Item = Entry<'a>>>
where
    L: IntoIterator<Item = Entry<'a>>,
    R: IntoIterator<Item = Entry<'a>>,
{
    let xor = Compare::xor(lhs, rhs).filter_map(option_xor);
    Pair(xor, PhantomData)
}

fn option_and<'a>(t: (Option<Entry<'a>>, Option<Entry<'a>>)) -> Option<Entry<'a>> {
    match t {
        (Some(mut lhs), Some(rhs)) => {
            lhs.cow.to_mut().and_assign(rhs.cow.as_ref());
            Some(lhs)
        }
        _ => None,
    }
}

fn option_or<'a>(t: (Option<Entry<'a>>, Option<Entry<'a>>)) -> Option<Entry<'a>> {
    match t {
        (Some(mut lhs), Some(rhs)) => {
            lhs.cow.to_mut().or_assign(rhs.cow.as_ref());
            Some(lhs)
        }
        (Some(lhs), None) => Some(lhs),
        (None, Some(rhs)) => Some(rhs),
        (None, None) => None,
    }
}

fn option_and_not<'a>(t: (Option<Entry<'a>>, Option<Entry<'a>>)) -> Option<Entry<'a>> {
    match t {
        (Some(mut lhs), Some(rhs)) => {
            lhs.cow.to_mut().and_not_assign(rhs.cow.as_ref());
            Some(lhs)
        }
        (Some(lhs), None) => Some(lhs),
        _ => None,
    }
}

fn option_xor<'a>(t: (Option<Entry<'a>>, Option<Entry<'a>>)) -> Option<Entry<'a>> {
    match t {
        (Some(mut lhs), Some(rhs)) => {
            lhs.cow.to_mut().xor_assign(rhs.cow.as_ref());
            Some(lhs)
        }
        (Some(lhs), None) => Some(lhs),
        (None, Some(rhs)) => Some(rhs),
        _ => None,
    }
}

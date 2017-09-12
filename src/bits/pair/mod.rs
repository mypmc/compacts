mod assign;
mod compare;
mod entry;

use std::marker::PhantomData;

pub use self::assign::Assign;
pub use self::compare::{Compare, Comparing};
pub use self::entry::{Entries, Entry};

mod sealed {
    pub trait Op {}
    pub struct And;
    pub struct Or;
    pub struct AndNot;
    pub struct Xor;
    impl Op for And {}
    impl Op for Or {}
    impl Op for AndNot {}
    impl Op for Xor {}
}

pub struct Pair<I, O: sealed::Op>(I, PhantomData<O>);

pub type And<I> = Pair<I, sealed::And>;
pub type Or<I> = Pair<I, sealed::Or>;
pub type AndNot<I> = Pair<I, sealed::AndNot>;
pub type Xor<I> = Pair<I, sealed::Xor>;

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
    let and = Compare::and(lhs, rhs).filter_map(entry::option_and);
    Pair(and, PhantomData)
}

pub fn or<'a, L, R>(lhs: L, rhs: R) -> Or<impl Iterator<Item = Entry<'a>>>
where
    L: IntoIterator<Item = Entry<'a>>,
    R: IntoIterator<Item = Entry<'a>>,
{
    let or = Compare::or(lhs, rhs).filter_map(entry::option_or);
    Pair(or, PhantomData)
}

pub fn and_not<'a, L, R>(lhs: L, rhs: R) -> AndNot<impl Iterator<Item = Entry<'a>>>
where
    L: IntoIterator<Item = Entry<'a>>,
    R: IntoIterator<Item = Entry<'a>>,
{
    let and_not = Compare::and_not(lhs, rhs).filter_map(entry::option_and_not);
    Pair(and_not, PhantomData)
}

pub fn xor<'a, L, R>(lhs: L, rhs: R) -> Xor<impl Iterator<Item = Entry<'a>>>
where
    L: IntoIterator<Item = Entry<'a>>,
    R: IntoIterator<Item = Entry<'a>>,
{
    let xor = Compare::xor(lhs, rhs).filter_map(entry::option_xor);
    Pair(xor, PhantomData)
}

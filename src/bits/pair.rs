use std::iter::{ExactSizeIterator, Fuse, Peekable};
use std::cmp::{self, Ordering};
use std::marker::PhantomData;

pub trait Intersection<Rhs = Self> {
    type Output;
    fn intersection(self, that: Rhs) -> Self::Output;
}
pub trait IntersectionWith<Rhs = Self> {
    fn intersection_with(&mut self, that: Rhs);
}

pub trait Union<Rhs = Self> {
    type Output;
    fn union(self, that: Rhs) -> Self::Output;
}
pub trait UnionWith<Rhs = Self> {
    fn union_with(&mut self, that: Rhs);
}

pub trait Difference<Rhs = Self> {
    type Output;
    fn difference(self, that: Rhs) -> Self::Output;
}
pub trait DifferenceWith<Rhs = Self> {
    fn difference_with(&mut self, that: Rhs);
}

pub trait SymmetricDifference<Rhs = Self> {
    type Output;
    fn symmetric_difference(self, that: Rhs) -> Self::Output;
}
pub trait SymmetricDifferenceWith<Rhs = Self> {
    fn symmetric_difference_with(&mut self, that: Rhs);
}

mod op {
    pub trait Sealed {}
    pub(crate) struct Intersection;
    pub(crate) struct Union;
    pub(crate) struct Difference;
    pub(crate) struct SymmetricDifference;
    impl Sealed for Intersection {}
    impl Sealed for Union {}
    impl Sealed for Difference {}
    impl Sealed for SymmetricDifference {}
}

pub(crate) struct Pair<I1, I2, Op>
where
    I1: Iterator + ExactSizeIterator,
    I2: Iterator + ExactSizeIterator,
    Op: op::Sealed,
{
    lhs: Peekable<Fuse<I1>>,
    rhs: Peekable<Fuse<I2>>,
    _op: PhantomData<Op>,
}

/// Compare `a` and `b`, but return `x` if a is None and `y` if b is None
fn comparing<T: Ord>(
    a: Option<T>,
    b: Option<T>,
    x: cmp::Ordering,
    y: cmp::Ordering,
) -> cmp::Ordering {
    match (a, b) {
        (None, _) => x,
        (_, None) => y,
        (Some(ref lhs), Some(ref rhs)) => lhs.cmp(rhs),
    }
}

impl<I1, I2, T> Iterator for Pair<I1, I2, op::Intersection>
where
    I1: Iterator<Item = T> + ExactSizeIterator,
    I2: Iterator<Item = T> + ExactSizeIterator,
    T: Ord,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        use self::Ordering::{Equal, Greater, Less};
        loop {
            let compared = {
                let x = self.lhs.peek();
                let y = self.rhs.peek();
                x.and_then(|x1| y.map(|y1| x1.cmp(y1)))
            };
            match compared {
                None => return None,
                Some(Less) => {
                    self.lhs.next();
                }
                Some(Equal) => {
                    self.rhs.next();
                    return self.lhs.next();
                }
                Some(Greater) => {
                    self.rhs.next();
                }
            }
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(cmp::min(self.lhs.len(), self.rhs.len())))
    }
}

impl<I1, I2, T> Iterator for Pair<I1, I2, op::Union>
where
    I1: Iterator<Item = T> + ExactSizeIterator,
    I2: Iterator<Item = T> + ExactSizeIterator,
    T: Ord,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        use self::Ordering::{Equal, Greater, Less};
        match comparing(self.lhs.peek(), self.rhs.peek(), Greater, Less) {
            Less => self.lhs.next(),
            Equal => {
                self.rhs.next();
                self.lhs.next()
            }
            Greater => self.rhs.next(),
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let x_len = self.lhs.len();
        let y_len = self.rhs.len();
        (cmp::max(x_len, y_len), Some(x_len + y_len))
    }
}

impl<I1, I2, T> Iterator for Pair<I1, I2, op::Difference>
where
    I1: Iterator<Item = T> + ExactSizeIterator,
    I2: Iterator<Item = T> + ExactSizeIterator,
    T: Ord,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        use self::Ordering::{Equal, Greater, Less};
        loop {
            let compaed = comparing(self.lhs.peek(), self.rhs.peek(), Less, Less);
            match compaed {
                Less => return self.lhs.next(),
                Equal => {
                    self.lhs.next();
                    self.rhs.next();
                }
                Greater => {
                    self.rhs.next();
                }
            }
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let x_len = self.lhs.len();
        let y_len = self.rhs.len();
        (x_len.saturating_sub(y_len), Some(x_len))
    }
}

impl<I1, I2, T> Iterator for Pair<I1, I2, op::SymmetricDifference>
where
    I1: Iterator<Item = T> + ExactSizeIterator,
    I2: Iterator<Item = T> + ExactSizeIterator,
    T: Ord,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        use self::Ordering::{Equal, Greater, Less};
        loop {
            match comparing(self.lhs.peek(), self.rhs.peek(), Greater, Less) {
                Less => return self.lhs.next(),
                Equal => {
                    self.lhs.next();
                    self.rhs.next();
                }
                Greater => return self.rhs.next(),
            }
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.lhs.len() + self.rhs.len()))
    }
}

macro_rules! pair_function {
    ( $fn:ident, $op:ident ) => (
        pub(crate) fn $fn<I1, I2, T>(x: I1, y: I2) -> Pair<I1, I2, op::$op>
            where I1: Iterator<Item = T> + ExactSizeIterator,
                  I2: Iterator<Item = T> + ExactSizeIterator,
                  T: Ord
        {
            Pair {
                lhs: x.fuse().peekable(),
                rhs: y.fuse().peekable(),
                _op: PhantomData,
            }
        }
    );
}

pair_function!(intersection, Intersection);
pair_function!(union, Union);
pair_function!(difference, Difference);
pair_function!(symmetric_difference, SymmetricDifference);

// Generic impl for Option.

impl<T1, T2> Intersection<Option<T2>> for Option<T1>
where
    T1: Intersection<T2>,
{
    type Output = Option<<T1 as Intersection<T2>>::Output>;
    fn intersection(self, that: Option<T2>) -> Self::Output {
        match (self, that) {
            (Some(t1), Some(t2)) => Some(t1.intersection(t2)),
            _ => None,
        }
    }
}

impl<T> Union<Option<T>> for Option<T>
where
    T: Union<T, Output = T>,
{
    type Output = Option<<T as Union>::Output>;
    fn union(self, that: Option<T>) -> Self::Output {
        match (self, that) {
            (Some(t1), Some(t2)) => Some(t1.union(t2)),
            (Some(t1), None) => Some(t1),
            (None, Some(t2)) => Some(t2),
            (None, None) => None,
        }
    }
}

impl<T1, T2> Difference<Option<T2>> for Option<T1>
where
    T1: Difference<T2, Output = T1>,
{
    type Output = Option<<T1 as Difference<T2>>::Output>;
    fn difference(self, that: Option<T2>) -> Self::Output {
        match (self, that) {
            (Some(t1), Some(t2)) => Some(t1.difference(t2)),
            (Some(t1), None) => Some(t1),
            _ => None,
        }
    }
}

impl<T> SymmetricDifference<Option<T>> for Option<T>
where
    T: SymmetricDifference<T, Output = T>,
{
    type Output = Option<<T as SymmetricDifference>::Output>;
    fn symmetric_difference(self, that: Option<T>) -> Self::Output {
        match (self, that) {
            (Some(t1), Some(t2)) => Some(t1.symmetric_difference(t2)),
            (Some(t1), None) => Some(t1),
            (None, Some(t2)) => Some(t2),
            _ => None,
        }
    }
}

pub trait Intersection<Rhs = Self> {
    type Output;
    fn intersection(self, that: Rhs) -> Self::Output;
}
pub trait Union<Rhs = Self> {
    type Output;
    fn union(self, that: Rhs) -> Self::Output;
}
pub trait Difference<Rhs = Self> {
    type Output;
    fn difference(self, that: Rhs) -> Self::Output;
}
pub trait SymmetricDifference<Rhs = Self> {
    type Output;
    fn symmetric_difference(self, that: Rhs) -> Self::Output;
}

impl<T> Intersection<Option<T>> for Option<T>
where
    T: Intersection<T, Output = T>,
{
    type Output = Option<T>;
    fn intersection(self, that: Option<T>) -> Self::Output {
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
    type Output = Option<T>;
    fn union(self, that: Option<T>) -> Self::Output {
        match (self, that) {
            (Some(t1), Some(t2)) => Some(t1.union(t2)),
            (Some(t1), None) => Some(t1),
            (None, Some(t2)) => Some(t2),
            (None, None) => None,
        }
    }
}

impl<'a, T> Difference<Option<T>> for Option<T>
where
    T: Difference<T, Output = T>,
{
    type Output = Option<T>;
    fn difference(self, that: Option<T>) -> Self::Output {
        match (self, that) {
            (Some(t1), Some(t2)) => Some(t1.difference(t2)),
            (Some(t1), None) => Some(t1),
            _ => None,
        }
    }
}

impl<'a, T> SymmetricDifference<Option<T>> for Option<T>
where
    T: SymmetricDifference<T, Output = T>,
{
    type Output = Option<T>;
    fn symmetric_difference(self, that: Option<T>) -> Self::Output {
        match (self, that) {
            (Some(t1), Some(t2)) => Some(t1.symmetric_difference(t2)),
            (Some(t1), None) => Some(t1),
            (None, Some(t2)) => Some(t2),
            _ => None,
        }
    }
}

pub trait IntersectionWith<Rhs = Self> {
    fn intersection_with(&mut self, that: Rhs);
}
pub trait UnionWith<Rhs = Self> {
    fn union_with(&mut self, that: Rhs);
}
pub trait DifferenceWith<Rhs = Self> {
    fn difference_with(&mut self, that: Rhs);
}
pub trait SymmetricDifferenceWith<Rhs = Self> {
    fn symmetric_difference_with(&mut self, that: Rhs);
}

impl<T> ::std::ops::BitAndAssign<T> for IntersectionWith<T> {
    fn bitand_assign(&mut self, that: T) {
        self.intersection_with(that)
    }
}
impl<T> ::std::ops::BitOrAssign<T> for UnionWith<T> {
    fn bitor_assign(&mut self, that: T) {
        self.union_with(that)
    }
}
impl<T> ::std::ops::SubAssign<T> for DifferenceWith<T> {
    fn sub_assign(&mut self, that: T) {
        self.difference_with(that);
    }
}
impl<T> ::std::ops::BitXorAssign<T> for SymmetricDifferenceWith<T> {
    fn bitxor_assign(&mut self, that: T) {
        self.symmetric_difference_with(that);
    }
}

macro_rules! impl_pairwise {
    ( $( $type:ty ),* ) => ($(
        impl IntersectionWith for $type {
            fn intersection_with(&mut self, rhs: $type) {*self &= rhs;}
        }
        impl UnionWith for $type {
            fn union_with(&mut self, rhs: $type) {*self |= rhs;}
        }
        impl DifferenceWith for $type {
            fn difference_with(&mut self, rhs: $type) {*self &= !rhs;}
        }
        impl SymmetricDifferenceWith for $type {
            fn symmetric_difference_with(&mut self, rhs: $type) {*self ^= rhs;}
        }
    )*)
}
impl_pairwise!(u8, u16, u32, u64, usize);

use std::iter::{Fuse, Peekable, ExactSizeIterator};
use std::cmp::{self, Ordering};

macro_rules! define_pair {
    ( $( ( $fn:ident, $op:ident ) ),* ) => ($(
        /// Struct for a slow but generic pairwise operations.
        pub struct $op<I1, I2, T>
            where I1: Iterator<Item = T>,
                  I2: Iterator<Item = T>
        {
            lhs: Peekable<Fuse<I1>>,
            rhs: Peekable<Fuse<I2>>,
        }

        /// Assume that I1 and I2 are sorted.
        pub fn $fn<I1, I2, T>(x: I1, y: I2) -> $op<I1, I2, T>
            where I1: Iterator<Item = T> + ExactSizeIterator,
                  I2: Iterator<Item = T> + ExactSizeIterator,
                  T: Ord
        {
            $op {lhs: x.fuse().peekable(), rhs: y.fuse().peekable()}
        }
    )*);
}

define_pair!(
    (intersection, IntersectionIter),
    (union, UnionIter),
    (difference, DifferenceIter),
    (symmetric_difference, SymmetricDifferenceIter)
);

/// Compare `a` and `b`, but return `s` if a is None and `l` if b is None
fn comparing<T: Ord>(
    a: Option<T>,
    b: Option<T>,
    x: cmp::Ordering,
    y: cmp::Ordering,
) -> cmp::Ordering {
    match (a, b) {
        (None, _) => x,
        (_, None) => y,
        (Some(ref a1), Some(ref b1)) => a1.cmp(b1),
    }
}

impl<I1, I2, T> Iterator for IntersectionIter<I1, I2, T>
where
    I1: Iterator<Item = T> + ExactSizeIterator,
    I2: Iterator<Item = T> + ExactSizeIterator,
    T: Ord,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        use self::Ordering::{Less, Equal, Greater};
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

impl<I1, I2, T> Iterator for UnionIter<I1, I2, T>
where
    I1: Iterator<Item = T> + ExactSizeIterator,
    I2: Iterator<Item = T> + ExactSizeIterator,
    T: Ord,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        use self::Ordering::{Less, Equal, Greater};
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

impl<I1, I2, T> Iterator for DifferenceIter<I1, I2, T>
where
    I1: Iterator<Item = T> + ExactSizeIterator,
    I2: Iterator<Item = T> + ExactSizeIterator,
    T: Ord,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        use self::Ordering::{Less, Equal, Greater};
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

impl<I1, I2, T> Iterator for SymmetricDifferenceIter<I1, I2, T>
where
    I1: Iterator<Item = T> + ExactSizeIterator,
    I2: Iterator<Item = T> + ExactSizeIterator,
    T: Ord,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        use self::Ordering::{Less, Equal, Greater};
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

#![allow(dead_code)]

use std::iter::{Peekable, IntoIterator};
use std::cmp::{self, Ordering};
use std::marker::PhantomData;

use self::Ordering::{Less, Equal, Greater};
use BucketIter;

pub struct Pair<'a, T> {
    lhs: Peekable<BucketIter<'a>>,
    rhs: Peekable<BucketIter<'a>>,
    _op: PhantomData<T>,
}

macro_rules! define_pair {
    ( $( ( $fn:ident, $op:ident ) ),* ) => ($(
        pub struct $op;
        impl<'a> Pair<'a, $op> {
            fn new(x: BucketIter<'a>, y: BucketIter<'a>) -> Pair<'a, $op> {
                Pair {
                    lhs: x.peekable(),
                    rhs: y.peekable(),
                    _op: PhantomData,
                }
            }
        }
        pub fn $fn<'a, I>(x: I, y: I) -> Pair<'a, $op>
            where I
            : IntoIterator<IntoIter = BucketIter<'a>>
            + IntoIterator<Item = <BucketIter<'a> as Iterator>::Item>
        {
            <Pair<'a, $op>>::new(x.into_iter(), y.into_iter())
        }
    )*);
}

define_pair!((intersection, Intersection),
             (union, Union),
             (difference, Difference),
             (symmetric_difference, SymmetricDifference));

/// Compare `a` and `b`, but return `s` if a is None and `l` if b is None
fn comparing<T: Ord>(a: Option<T>, b: Option<T>, x: Ordering, y: Ordering) -> Ordering {
    match (a, b) {
        (None, _) => x,
        (_, None) => y,
        (Some(ref a1), Some(ref b1)) => a1.cmp(b1),
    }
}

impl<'a> Iterator for Pair<'a, Intersection> {
    type Item = u16;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match {
                      let x = self.lhs.peek();
                      let y = self.rhs.peek();
                      x.and_then(|x1| y.map(|y1| x1.cmp(&y1)))
                  } {
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

impl<'a> Iterator for Pair<'a, Union> {
    type Item = u16;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match comparing(self.lhs.peek(), self.rhs.peek(), Greater, Less) {
                Less => return self.lhs.next(),
                Equal => {
                    self.rhs.next();
                    return self.lhs.next();
                }
                Greater => return self.rhs.next(),
            }
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let x_len = self.lhs.len();
        let y_len = self.rhs.len();
        (cmp::max(x_len, y_len), Some(x_len + y_len))
    }
}

impl<'a> Iterator for Pair<'a, Difference> {
    type Item = u16;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match comparing(self.lhs.peek(), self.rhs.peek(), Less, Less) {
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

impl<'a> Iterator for Pair<'a, SymmetricDifference> {
    type Item = u16;
    fn next(&mut self) -> Option<Self::Item> {
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

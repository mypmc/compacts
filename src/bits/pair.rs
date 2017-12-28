use std::iter::Peekable;
use std::cmp::{self, Ordering};
use self::Ordering::{Equal as EQ, Greater as GT, Less as LT};

macro_rules! defops {
    ( $( ($tyname:ident, $fnname:ident) ),* ) => ($(
        pub struct $tyname<L: Iterator, R: Iterator> {
            lhs: Peekable<L>,
            rhs: Peekable<R>,
        }
        pub fn $fnname<L, R>(lhs: L, rhs: R) -> $tyname<L::IntoIter, R::IntoIter>
        where
            L: IntoIterator,
            R: IntoIterator,
        {
            $tyname {
                lhs: lhs.into_iter().peekable(),
                rhs: rhs.into_iter().peekable(),
            }
        }
    )*)
}
defops!((And, and), (Or, or), (AndNot, and_not), (Xor, xor));

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

impl<L, R, T> Iterator for And<L, R>
where
    L: Iterator<Item = T>,
    R: Iterator<Item = T>,
    T: Ord,
{
    type Item = (Option<L::Item>, Option<R::Item>);
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let compared = {
                if let Some(x) = self.lhs.peek() {
                    self.rhs.peek().map(|y| x.cmp(y))
                } else {
                    None
                }
            };
            match compared {
                None => break None,
                Some(LT) => {
                    self.lhs.next();
                }
                Some(EQ) => {
                    let lhs = self.lhs.next();
                    let rhs = self.rhs.next();
                    break Some((lhs, rhs));
                }
                Some(GT) => {
                    self.rhs.next();
                }
            }
        }
    }
}

impl<L, R, T> Iterator for Or<L, R>
where
    L: Iterator<Item = T>,
    R: Iterator<Item = T>,
    T: Ord,
{
    type Item = (Option<L::Item>, Option<R::Item>);
    fn next(&mut self) -> Option<Self::Item> {
        match comparing(self.lhs.peek(), self.rhs.peek(), GT, LT) {
            LT => self.lhs.next().map(|lhs| (Some(lhs), None)),
            EQ => {
                let lhs = self.lhs.next();
                let rhs = self.rhs.next();
                Some((lhs, rhs))
            }
            GT => self.rhs.next().map(|rhs| (None, Some(rhs))),
        }
    }
}

impl<L, R, T> Iterator for AndNot<L, R>
where
    L: Iterator<Item = T>,
    R: Iterator<Item = T>,
    T: Ord,
{
    type Item = (Option<L::Item>, Option<R::Item>);
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match comparing(self.lhs.peek(), self.rhs.peek(), LT, LT) {
                LT => break self.lhs.next().map(|lhs| (Some(lhs), None)),
                EQ => {
                    let lhs = self.lhs.next();
                    let rhs = self.rhs.next();
                    break Some((lhs, rhs));
                }
                GT => {
                    self.rhs.next();
                }
            }
        }
    }
}

impl<L, R, T> Iterator for Xor<L, R>
where
    L: Iterator<Item = T>,
    R: Iterator<Item = T>,
    T: Ord,
{
    type Item = (Option<L::Item>, Option<R::Item>);
    fn next(&mut self) -> Option<Self::Item> {
        match comparing(self.lhs.peek(), self.rhs.peek(), GT, LT) {
            LT => self.lhs.next().map(|lhs| (Some(lhs), None)),
            EQ => {
                let lhs = self.lhs.next();
                let rhs = self.rhs.next();
                Some((lhs, rhs))
            }
            GT => self.rhs.next().map(|rhs| (None, Some(rhs))),
        }
    }
}

pub struct Merge<L, R>
where
    L: Iterator,
    R: Iterator,
{
    lhs: Peekable<L>,
    rhs: Peekable<R>,
}

impl<L, R, T> Iterator for Merge<L, R>
where
    L: Iterator<Item = T>,
    R: Iterator<Item = T>,
    T: Ord,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        match comparing(self.lhs.peek(), self.rhs.peek(), GT, LT) {
            LT | EQ => self.lhs.next(),
            GT => self.rhs.next(),
        }
    }
}

pub fn merge<L, R, T>(lhs: L, rhs: R) -> Merge<L::IntoIter, R::IntoIter>
where
    L: IntoIterator<Item = T>,
    R: IntoIterator<Item = T>,
    T: Ord,
{
    let lhs = lhs.into_iter().peekable();
    let rhs = rhs.into_iter().peekable();
    Merge { lhs, rhs }
}

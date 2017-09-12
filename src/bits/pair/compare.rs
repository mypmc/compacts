use std::iter::Peekable;
use std::cmp::{self, Ordering};
use std::marker::PhantomData;

use super::sealed;
use self::Ordering::{Equal as EQ, Greater as GT, Less as LT};

pub struct Comparing<L, R, T, O>
where
    L: Iterator<Item = T>,
    R: Iterator<Item = T>,
    O: sealed::Op,
{
    lhs: Peekable<L>,
    rhs: Peekable<R>,
    _op: PhantomData<O>,
}

pub struct Compare;

impl Compare {
    pub fn and<L, R, T>(
        lhs: L,
        rhs: R,
    ) -> Comparing<impl Iterator<Item = T>, impl Iterator<Item = T>, T, sealed::And>
    where
        L: IntoIterator<Item = T>,
        R: IntoIterator<Item = T>,
    {
        Comparing {
            lhs: lhs.into_iter().peekable(),
            rhs: rhs.into_iter().peekable(),
            _op: PhantomData,
        }
    }

    pub fn or<L, R, T>(
        lhs: L,
        rhs: R,
    ) -> Comparing<impl Iterator<Item = T>, impl Iterator<Item = T>, T, sealed::Or>
    where
        L: IntoIterator<Item = T>,
        R: IntoIterator<Item = T>,
    {
        Comparing {
            lhs: lhs.into_iter().peekable(),
            rhs: rhs.into_iter().peekable(),
            _op: PhantomData,
        }
    }

    pub fn and_not<L, R, T>(
        lhs: L,
        rhs: R,
    ) -> Comparing<impl Iterator<Item = T>, impl Iterator<Item = T>, T, sealed::AndNot>
    where
        L: IntoIterator<Item = T>,
        R: IntoIterator<Item = T>,
    {
        Comparing {
            lhs: lhs.into_iter().peekable(),
            rhs: rhs.into_iter().peekable(),
            _op: PhantomData,
        }
    }

    pub fn xor<L, R, T>(
        lhs: L,
        rhs: R,
    ) -> Comparing<impl Iterator<Item = T>, impl Iterator<Item = T>, T, sealed::Xor>
    where
        L: IntoIterator<Item = T>,
        R: IntoIterator<Item = T>,
    {
        Comparing {
            lhs: lhs.into_iter().peekable(),
            rhs: rhs.into_iter().peekable(),
            _op: PhantomData,
        }
    }
}

impl<L, R, T> Iterator for Comparing<L, R, T, sealed::And>
where
    L: Iterator<Item = T>,
    R: Iterator<Item = T>,
    T: Ord,
{
    type Item = (Option<T>, Option<T>);
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let compared = {
                let optx = self.lhs.peek();
                let opty = self.rhs.peek();
                optx.and_then(|x| opty.map(|y| x.cmp(y)))
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

impl<L, R, T> Iterator for Comparing<L, R, T, sealed::Or>
where
    L: Iterator<Item = T>,
    R: Iterator<Item = T>,
    T: Ord,
{
    type Item = (Option<T>, Option<T>);

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

    // fn size_hint(&self) -> (usize, Option<usize>) {
    //     let (_, lhs) = self.lhs.size_hint();
    //     let (_, rhs) = self.rhs.size_hint();
    //     if lhs.is_some() && rhs.is_some() {
    //         (0, Some(cmp::min(lhs.unwrap(), rhs.unwrap())))
    //     } else {
    //         (0, None)
    //     }
    //     // (0, Some(cmp::min(self.lhs.len(), self.rhs.len())))
    // }
}

impl<L, R, T> Iterator for Comparing<L, R, T, sealed::AndNot>
where
    L: Iterator<Item = T>,
    R: Iterator<Item = T>,
    T: Ord,
{
    type Item = (Option<T>, Option<T>);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match comparing(self.lhs.peek(), self.rhs.peek(), LT, LT) {
                LT => break self.lhs.next().map(|lhs| (Some(lhs), None)),
                EQ => {
                    let lhs = self.lhs.next();
                    let rhs = self.rhs.next();
                    assert!(lhs.is_some() && rhs.is_some());
                    break Some((lhs, rhs));
                }
                GT => {
                    self.rhs.next();
                }
            }
        }
    }
}

impl<L, R, T> Iterator for Comparing<L, R, T, sealed::Xor>
where
    L: Iterator<Item = T>,
    R: Iterator<Item = T>,
    T: Ord,
{
    type Item = (Option<T>, Option<T>);

    fn next(&mut self) -> Option<Self::Item> {
        match comparing(self.lhs.peek(), self.rhs.peek(), GT, LT) {
            LT => self.lhs.next().map(|lhs| (Some(lhs), None)),
            EQ => {
                let lhs = self.lhs.next();
                let rhs = self.rhs.next();
                assert!(lhs.is_some() && rhs.is_some());
                Some((lhs, rhs))
            }
            GT => self.rhs.next().map(|rhs| (None, Some(rhs))),
        }
    }
}

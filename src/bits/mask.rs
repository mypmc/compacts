use std::{
    borrow::Cow,
    cmp::Ordering,
    iter::Peekable,
    marker::PhantomData,
    ops::{BitAndAssign, BitOrAssign, BitXorAssign},
};

use crate::bits::{Page, UnsignedInt};

mod sealed {
    pub trait Op {}

    #[derive(Debug)]
    pub struct And;
    #[derive(Debug)]
    pub struct Or;
    #[derive(Debug)]
    pub struct Xor;

    impl Op for And {}
    impl Op for Or {}
    impl Op for Xor {}
}
use sealed::Op;

#[derive(Debug)]
pub struct Mask<L, R, O: Op> {
    lhs: L,
    rhs: R,
    _op: PhantomData<O>,
}

pub type And<L, R> = Mask<L, R, sealed::And>;
pub type Or<L, R> = Mask<L, R, sealed::Or>;
pub type Xor<L, R> = Mask<L, R, sealed::Xor>;

impl<L, R, O: Op> Mask<L, R, O> {
    fn mask(lhs: L, rhs: R) -> Self {
        Mask {
            lhs,
            rhs,
            _op: PhantomData,
        }
    }

    pub fn and<Rhs>(self, rhs: Rhs) -> And<Self, Rhs> {
        and(self, rhs)
    }

    pub fn or<Rhs>(self, rhs: Rhs) -> Or<Self, Rhs> {
        or(self, rhs)
    }

    pub fn xor<Rhs>(self, rhs: Rhs) -> Xor<Self, Rhs> {
        xor(self, rhs)
    }
}

/// ```
/// use compacts::bits;
/// let a = vec![0b_00001111_u8, 0b_10101010_u8];
/// let b = vec![0b_11110000_u8, 0b_01010101_u8];
/// let r = bits::and(a, b).into_iter().collect::<Vec<_>>();
/// assert_eq!(r, vec![0, 0]);
/// ```
pub fn and<L, R>(lhs: L, rhs: R) -> And<L, R> {
    Mask::mask(lhs, rhs)
}

/// ```
/// use compacts::bits;
/// let a = vec![0b_00001111_u8, 0b_10101010_u8];
/// let b = vec![0b_11110000_u8, 0b_01010101_u8];
/// let r = bits::or(a, b).into_iter().collect::<Vec<_>>();
/// assert_eq!(r, vec![!0, !0]);
/// ```
pub fn or<L, R>(lhs: L, rhs: R) -> Or<L, R> {
    Mask::mask(lhs, rhs)
}

/// ```
/// use compacts::bits;
/// let a = vec![0b_11001100_u8, 0b_11110000_u8];
/// let b = vec![0b_11110000_u8, 0b_01010101_u8];
/// let r = bits::xor(a, b).into_iter().collect::<Vec<_>>();
/// assert_eq!(r, vec![0b_00111100, 0b_10100101]);
/// ```
pub fn xor<L, R>(lhs: L, rhs: R) -> Xor<L, R> {
    Mask::mask(lhs, rhs)
}

pub struct Iter<L: Iterator, R: Iterator, T, O: Op> {
    lhs: Peekable<L>,
    rhs: Peekable<R>,
    _ty: PhantomData<T>,
    _op: PhantomData<O>,
}

impl<L, R, T, O> IntoIterator for Mask<L, R, O>
where
    L: IntoIterator<Item = T>,
    R: IntoIterator<Item = T>,
    O: Op,
    Iter<L::IntoIter, R::IntoIter, T, O>: Iterator,
{
    type Item = <Iter<L::IntoIter, R::IntoIter, T, O> as Iterator>::Item;
    type IntoIter = Iter<L::IntoIter, R::IntoIter, T, O>;
    fn into_iter(self) -> Self::IntoIter {
        Iter {
            lhs: self.lhs.into_iter().peekable(),
            rhs: self.rhs.into_iter().peekable(),
            _ty: PhantomData,
            _op: PhantomData,
        }
    }
}

impl<L, R, V> Iterator for Iter<L, R, V, sealed::And>
where
    L: Iterator<Item = V>,
    R: Iterator<Item = V>,
    V: UnsignedInt,
{
    type Item = V;
    fn next(&mut self) -> Option<Self::Item> {
        self.lhs.next().and_then(|x| self.rhs.next().map(|y| x & y))
    }
}

impl<L, R, V> Iterator for Iter<L, R, V, sealed::Or>
where
    L: Iterator<Item = V>,
    R: Iterator<Item = V>,
    V: UnsignedInt,
{
    type Item = V;
    fn next(&mut self) -> Option<Self::Item> {
        match (self.lhs.next(), self.rhs.next()) {
            (Some(x), Some(y)) => Some(x | y),
            (Some(x), None) => Some(x),
            (None, Some(y)) => Some(y),
            (None, None) => None,
        }
    }
}

impl<L, R, V> Iterator for Iter<L, R, V, sealed::Xor>
where
    L: Iterator<Item = V>,
    R: Iterator<Item = V>,
    V: UnsignedInt,
{
    type Item = V;
    fn next(&mut self) -> Option<Self::Item> {
        match (self.lhs.next(), self.rhs.next()) {
            (Some(x), Some(y)) => Some(x ^ y),
            (Some(x), None) => Some(x),
            (None, Some(y)) => Some(y),
            (None, None) => None,
        }
    }
}

impl<'a, L, R, V> Iterator for Iter<L, R, Cow<'a, V>, sealed::And>
where
    L: Iterator<Item = Cow<'a, V>>,
    R: Iterator<Item = Cow<'a, V>>,
    V: BitAndAssign<Cow<'a, V>> + Clone + 'a,
{
    type Item = Cow<'a, V>;
    fn next(&mut self) -> Option<Self::Item> {
        let lhs = &mut self.lhs;
        let rhs = &mut self.rhs;
        lhs.next().and_then(|mut x| {
            rhs.next().map(|y| {
                x.to_mut().bitand_assign(y);
                x
            })
        })
    }
}

impl<'a, L, R, V> Iterator for Iter<L, R, Cow<'a, V>, sealed::Or>
where
    L: Iterator<Item = Cow<'a, V>>,
    R: Iterator<Item = Cow<'a, V>>,
    V: BitOrAssign<Cow<'a, V>> + Clone + 'a,
{
    type Item = Cow<'a, V>;
    fn next(&mut self) -> Option<Self::Item> {
        match (self.lhs.next(), self.rhs.next()) {
            (Some(mut x), Some(y)) => {
                x.to_mut().bitor_assign(y);
                Some(x)
            }
            (Some(x), None) => Some(x),
            (None, Some(y)) => Some(y),
            (None, None) => None,
        }
    }
}

impl<'a, L, R, V> Iterator for Iter<L, R, Cow<'a, V>, sealed::Xor>
where
    L: Iterator<Item = Cow<'a, V>>,
    R: Iterator<Item = Cow<'a, V>>,
    V: BitXorAssign<Cow<'a, V>> + Clone + 'a,
{
    type Item = Cow<'a, V>;
    fn next(&mut self) -> Option<Self::Item> {
        match (self.lhs.next(), self.rhs.next()) {
            (Some(mut x), Some(y)) => {
                x.to_mut().bitxor_assign(y);
                Some(x)
            }
            (Some(x), None) => Some(x),
            (None, Some(y)) => Some(y),
            (None, None) => None,
        }
    }
}

impl<'a, L, R, K, V> Iterator for Iter<L, R, Page<K, Cow<'a, V>>, sealed::And>
where
    L: Iterator<Item = Page<K, Cow<'a, V>>>,
    R: Iterator<Item = Page<K, Cow<'a, V>>>,
    K: UnsignedInt,
    V: BitAndAssign<Cow<'a, V>> + Clone + 'a,
{
    type Item = Page<K, Cow<'a, V>>;
    fn next(&mut self) -> Option<Self::Item> {
        let lhs = &mut self.lhs;
        let rhs = &mut self.rhs;

        loop {
            match lhs
                .peek()
                .and_then(|x| rhs.peek().map(|y| x.index.cmp(&y.index)))
            {
                Some(Ordering::Equal) => {
                    let mut lhs = lhs.next().unwrap();
                    let rhs = rhs.next().unwrap();
                    lhs.value.to_mut().bitand_assign(rhs.value);
                    break Some(lhs);
                }
                Some(Ordering::Less) => {
                    lhs.next();
                }
                Some(Ordering::Greater) => {
                    rhs.next();
                }
                None => break None,
            }
        }
    }
}

impl<'a, L, R, K, V> Iterator for Iter<L, R, Page<K, Cow<'a, V>>, sealed::Or>
where
    L: Iterator<Item = Page<K, Cow<'a, V>>>,
    R: Iterator<Item = Page<K, Cow<'a, V>>>,
    K: UnsignedInt,
    V: BitOrAssign<Cow<'a, V>> + Clone + 'a,
{
    type Item = Page<K, Cow<'a, V>>;
    fn next(&mut self) -> Option<Self::Item> {
        let lhs = &mut self.lhs;
        let rhs = &mut self.rhs;

        match (lhs.peek(), rhs.peek()) {
            (Some(l), Some(r)) => match l.index.cmp(&r.index) {
                Ordering::Less => lhs.next(),
                Ordering::Equal => {
                    let mut lhs = lhs.next().expect("unreachable");
                    let rhs = rhs.next().expect("unreachable");
                    lhs.value.to_mut().bitor_assign(rhs.value);
                    Some(lhs)
                }
                Ordering::Greater => rhs.next(),
            },
            (Some(_), None) => lhs.next(),
            (None, Some(_)) => rhs.next(),
            (None, None) => None,
        }
    }
}

impl<'a, L, R, K, V> Iterator for Iter<L, R, Page<K, Cow<'a, V>>, sealed::Xor>
where
    L: Iterator<Item = Page<K, Cow<'a, V>>>,
    R: Iterator<Item = Page<K, Cow<'a, V>>>,
    K: UnsignedInt,
    V: BitXorAssign<Cow<'a, V>> + Clone + 'a,
{
    type Item = Page<K, Cow<'a, V>>;
    fn next(&mut self) -> Option<Self::Item> {
        let lhs = &mut self.lhs;
        let rhs = &mut self.rhs;

        match (lhs.peek(), rhs.peek()) {
            (Some(l), Some(r)) => match l.index.cmp(&r.index) {
                Ordering::Less => lhs.next(),
                Ordering::Equal => {
                    let mut lhs = lhs.next().expect("unreachable");
                    let rhs = rhs.next().expect("unreachable");
                    lhs.value.to_mut().bitxor_assign(rhs.value);
                    Some(lhs)
                }
                Ordering::Greater => rhs.next(),
            },
            (Some(_), None) => lhs.next(),
            (None, Some(_)) => rhs.next(),
            (None, None) => None,
        }
    }
}

pub struct Fold<'a, T>(Option<BoxIter<'a, T>>);

type BoxIter<'a, T> = Box<dyn Iterator<Item = T> + 'a>;

impl<'a, T: 'a> Iterator for Fold<'a, T> {
    type Item = T;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.as_mut().and_then(|it| it.next())
    }
}

impl<'a, T: 'a> Fold<'a, T> {
    /// Combines all given iterators into one iterator by using `And`.
    ///
    /// # Examples
    ///
    /// ```
    /// use compacts::bits::Fold;
    /// let a = vec![0b_11001010_u8, 0b_11110101];
    /// let b = vec![0b_11001100_u8, 0b_01010101];
    /// let c = vec![0b_11001011_u8, 0b_11000111];
    ///
    /// let out = Fold::and(vec![
    ///     a.iter().cloned(),
    ///     b.iter().cloned(),
    ///     c.iter().cloned(),
    /// ]).collect::<Vec<_>>();
    /// assert_eq!(out, vec![0b_11001000, 0b_01000101]);
    /// ```
    pub fn and<U>(iters: impl IntoIterator<Item = U>) -> Self
    where
        U: IntoIterator<Item = T> + 'a,
        And<BoxIter<'a, T>, U>: IntoIterator<Item = T>,
    {
        Self::fold(iters, and)
    }

    pub fn or<U>(iters: impl IntoIterator<Item = U>) -> Self
    where
        U: IntoIterator<Item = T> + 'a,
        Or<BoxIter<'a, T>, U>: IntoIterator<Item = T>,
    {
        Self::fold(iters, or)
    }

    pub fn xor<U>(iters: impl IntoIterator<Item = U>) -> Self
    where
        U: IntoIterator<Item = T> + 'a,
        Xor<BoxIter<'a, T>, U>: IntoIterator<Item = T>,
    {
        Self::fold(iters, xor)
    }

    fn fold<A, B>(iters: impl IntoIterator<Item = A>, func: impl Fn(BoxIter<'a, T>, A) -> B) -> Self
    where
        A: IntoIterator<Item = T> + 'a,
        B: IntoIterator<Item = T> + 'a,
    {
        let mut iters = iters.into_iter();
        Fold(if let Some(head) = iters.next() {
            let head = Box::new(head.into_iter());
            Some(iters.fold(head, |it, x| Box::new(func(it, x).into_iter())))
        } else {
            None
        })
    }
}

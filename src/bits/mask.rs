//! Module mask provides *bitwise* operations.

use std::{
    borrow::Cow,
    cmp::Ordering::{self, Equal, Greater, Less},
    iter::{empty, Peekable},
};

/// A trait for bitwise masking.
pub trait Mask<'a>: Sized {
    /// `Block` is an unit of bitwise operations.
    type Block: 'a + ?Sized + ToOwned;

    /// `Steps` yields Clone-On-Write `Block` with its index.
    type Steps: Iterator<Item = (usize, Cow<'a, Self::Block>)>;

    /// An iterator over bit blocks.
    fn into_steps(self) -> Self::Steps;

    /// Returns an iterator that performs bitwise intersection.
    fn and<Rhs: Mask<'a>>(self, that: Rhs) -> And<'a, Self, Rhs> {
        And::new(self, that)
    }

    /// Returns an iterator that performs bitwise union.
    fn or<Rhs: Mask<'a>>(self, that: Rhs) -> Or<'a, Self, Rhs> {
        Or::new(self, that)
    }

    /// Returns an iterator that performs bitwise difference.
    fn and_not<Rhs: Mask<'a>>(self, that: Rhs) -> AndNot<'a, Self, Rhs> {
        AndNot::new(self, that)
    }

    /// Returns an iterator that performs bitwise symmetric difference.
    fn xor<Rhs: Mask<'a>>(self, that: Rhs) -> Xor<'a, Self, Rhs> {
        Xor::new(self, that)
    }
}

// upstream crates may add new impl of trait `std::iter::Iterator` for type `&[_]` in future versions
impl<'a, I, T> Mask<'a> for I
where
    T: 'a + ?Sized + ToOwned,
    I: IntoIterator<Item = (usize, Cow<'a, T>)>,
{
    type Block = T;
    type Steps = I::IntoIter;
    fn into_steps(self) -> Self::Steps {
        self.into_iter()
    }
}

macro_rules! defops {
    ( $( $name:ident ),* ) => ($(
        /// An iterator that iterates two other iterators `block`wisely.
        ///
        /// Requirements for this struct to be a `Mask`
        ///
        /// 1) Both `L` and `R` implement `Mask`.
        ///
        /// 2) Both `L` and `R` have the same `Block` type.
        #[must_use = "do nothing unless consumed"]
        pub struct $name<'a, L: Mask<'a>, R: Mask<'a>> {
            lhs: Peekable<L::Steps>,
            rhs: Peekable<R::Steps>,
        }

        impl<'a, L: Mask<'a>, R: Mask<'a>> $name<'a, L, R> {
            pub(crate) fn new(lhs: L, rhs: R) -> Self {
                $name {
                    lhs: lhs.into_steps().peekable(),
                    rhs: rhs.into_steps().peekable(),
                }
            }
        }
    )*);
}
defops!(And, AndNot, Or, Xor);

impl<'a, L, R> Iterator for And<'a, L, R>
where
    L: Mask<'a>,
    R: Mask<'a, Block = L::Block>,
    <L::Block as ToOwned>::Owned: Intersection<L::Block>,
{
    type Item = (usize, Cow<'a, L::Block>);
    fn next(&mut self) -> Option<Self::Item> {
        let lhs = &mut self.lhs;
        let rhs = &mut self.rhs;
        loop {
            let compared = lhs
                .peek()
                .and_then(|(x, _)| rhs.peek().map(|(y, _)| x.cmp(y)));

            match compared {
                Some(Less) => {
                    lhs.next();
                }
                Some(Equal) => {
                    let (i, mut lhs) = lhs.next().expect("unreachable");
                    let (j, rhs) = rhs.next().expect("unreachable");
                    debug_assert_eq!(i, j);
                    lhs.to_mut().intersection(&rhs);
                    break Some((i, lhs));
                }
                Some(Greater) => {
                    rhs.next();
                }
                None => break None,
            }
        }
    }
}

impl<'a, L, R> Iterator for Or<'a, L, R>
where
    L: Mask<'a>,
    R: Mask<'a, Block = L::Block>,
    <L::Block as ToOwned>::Owned: Union<L::Block>,
{
    type Item = (usize, Cow<'a, L::Block>);
    fn next(&mut self) -> Option<Self::Item> {
        let lhs = &mut self.lhs;
        let rhs = &mut self.rhs;
        match cmp_index(lhs.peek(), rhs.peek(), Greater, Less) {
            Less => lhs.next(),
            Equal => {
                let (i, mut lhs) = lhs.next().expect("unreachable");
                let (j, rhs) = rhs.next().expect("unreachable");
                debug_assert_eq!(i, j);
                lhs.to_mut().union(rhs.as_ref());
                Some((i, lhs))
            }
            Greater => rhs.next(),
        }
    }
}

impl<'a, L, R> Iterator for AndNot<'a, L, R>
where
    L: Mask<'a>,
    R: Mask<'a, Block = L::Block>,
    <L::Block as ToOwned>::Owned: Difference<L::Block>,
{
    type Item = (usize, Cow<'a, L::Block>);
    fn next(&mut self) -> Option<Self::Item> {
        let lhs = &mut self.lhs;
        let rhs = &mut self.rhs;
        loop {
            match cmp_index(lhs.peek(), rhs.peek(), Less, Less) {
                Less => return lhs.next(),
                Equal => {
                    let (i, mut lhs) = lhs.next().expect("unreachable");
                    let (j, rhs) = rhs.next().expect("unreachable");
                    debug_assert_eq!(i, j);
                    lhs.to_mut().difference(rhs.as_ref());
                    return Some((i, lhs));
                }
                Greater => {
                    rhs.next();
                }
            };
        }
    }
}

impl<'a, L, R> Iterator for Xor<'a, L, R>
where
    L: Mask<'a>,
    R: Mask<'a, Block = L::Block>,
    <L::Block as ToOwned>::Owned: SymmetricDifference<L::Block>,
{
    type Item = (usize, Cow<'a, L::Block>);
    fn next(&mut self) -> Option<Self::Item> {
        let lhs = &mut self.lhs;
        let rhs = &mut self.rhs;
        match cmp_index(lhs.peek(), rhs.peek(), Greater, Less) {
            Less => lhs.next(),
            Equal => {
                let (i, mut lhs) = lhs.next().expect("unreachable");
                let (j, rhs) = rhs.next().expect("unreachable");
                debug_assert_eq!(i, j);
                lhs.to_mut().symmetric_difference(rhs.as_ref());
                Some((i, lhs))
            }
            Greater => rhs.next(),
        }
    }
}

/// The bitwise intersection.
#[inline]
pub fn and<'a, L: Mask<'a>, R: Mask<'a>>(lhs: L, rhs: R) -> And<'a, L, R> {
    And::new(lhs, rhs)
}

/// The bitwise union.
#[inline]
pub fn or<'a, L: Mask<'a>, R: Mask<'a>>(lhs: L, rhs: R) -> Or<'a, L, R> {
    Or::new(lhs, rhs)
}

/// The bitwise difference.
#[inline]
pub fn and_not<'a, L: Mask<'a>, R: Mask<'a>>(lhs: L, rhs: R) -> AndNot<'a, L, R> {
    AndNot::new(lhs, rhs)
}

/// The bitwise symmetric difference.
#[inline]
pub fn xor<'a, L: Mask<'a>, R: Mask<'a>>(lhs: L, rhs: R) -> Xor<'a, L, R> {
    Xor::new(lhs, rhs)
}

/// The bitwise in-place intersection.
pub trait Intersection<T: ?Sized> {
    /// Performs in-place intersection.
    fn intersection(&mut self, data: &T);
}

/// The bitwise in-place union.
pub trait Union<T: ?Sized> {
    /// Performs in-place union.
    fn union(&mut self, data: &T);
}

/// The bitwise in-place difference.
pub trait Difference<T: ?Sized> {
    /// Performs in-place difference.
    fn difference(&mut self, data: &T);
}

/// The bitwise in-place symmetric difference.
pub trait SymmetricDifference<T: ?Sized> {
    /// Performs in-place symmetric difference.
    fn symmetric_difference(&mut self, data: &T);
}

fn cmp_index<T>(
    x: Option<&(usize, T)>,
    y: Option<&(usize, T)>,
    none_x: Ordering,
    none_y: Ordering,
) -> Ordering {
    match (x, y) {
        (None, _) => none_x,
        (_, None) => none_y,
        (Some((i, _)), Some((j, _))) => i.cmp(j),
    }
}

/// `Fold` is an iterator built from `Mask`s.
pub struct Fold<'a, T>(Box<dyn Iterator<Item = (usize, T)> + 'a>);

impl<'a, T: ?Sized> Fold<'a, Cow<'a, T>>
where
    T: 'a + ToOwned,
{
    pub(crate) fn fold<A, B, F>(xs: impl IntoIterator<Item = A>, mut f: F) -> Fold<'a, Cow<'a, T>>
    where
        A: 'a + Mask<'a, Block = T>,
        B: 'a + Mask<'a, Block = T>,
        F: FnMut(Box<dyn Iterator<Item = (usize, Cow<'a, T>)> + 'a>, A) -> B,
    {
        let mut xs = xs.into_iter();
        if let Some(head) = xs.next() {
            let init = Box::new(head.into_steps());
            Fold(xs.fold(init, |a, x| Box::new(f(a, x).into_steps())))
        } else {
            Fold(Box::new(empty()))
        }
    }

    /// Folds `xs` into a single iterator that applies `and` to each bits.
    pub fn and<A>(xs: impl IntoIterator<Item = A>) -> Self
    where
        A: Mask<'a, Block = T>,
        And<'a, Box<dyn Iterator<Item = (usize, Cow<'a, T>)> + 'a>, A>: 'a + Mask<'a, Block = T>,
    {
        Self::fold(xs, And::new)
    }

    /// Folds `xs` into a single iterator that applies `or` to each bits.
    pub fn or<A>(xs: impl IntoIterator<Item = A>) -> Self
    where
        A: Mask<'a, Block = T>,
        Or<'a, Box<dyn Iterator<Item = (usize, Cow<'a, T>)> + 'a>, A>: 'a + Mask<'a, Block = T>,
    {
        Self::fold(xs, Or::new)
    }

    /// Folds `xs` into a single iterator that applies `and_not` to each bits.
    pub fn and_not<A>(xs: impl IntoIterator<Item = A>) -> Self
    where
        A: Mask<'a, Block = T>,
        AndNot<'a, Box<dyn Iterator<Item = (usize, Cow<'a, T>)> + 'a>, A>: 'a + Mask<'a, Block = T>,
    {
        Self::fold(xs, AndNot::new)
    }

    /// Folds `xs` into a single iterator that applies `xor` to each bits.
    pub fn xor<A>(xs: impl IntoIterator<Item = A>) -> Self
    where
        A: Mask<'a, Block = T>,
        Xor<'a, Box<dyn Iterator<Item = (usize, Cow<'a, T>)> + 'a>, A>: 'a + Mask<'a, Block = T>,
    {
        Self::fold(xs, Xor::new)
    }
}

impl<'a, T: 'a + ?Sized + ToOwned> Iterator for Fold<'a, Cow<'a, T>> {
    type Item = (usize, Cow<'a, T>);
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

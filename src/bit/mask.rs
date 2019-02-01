use std::{
    borrow::Cow,
    cmp::Ordering,
    iter::Peekable,
    marker::PhantomData,
    ops::{BitAndAssign, BitOrAssign, BitXorAssign},
};

use crate::bit;

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

    //     pub fn not(self) -> Not<Self> {
    //         not(self)
    //     }
}

// impl<I> Not<I> {
//     pub fn and<Rhs>(self, rhs: Rhs) -> And<Self, Rhs> {
//         and(self, rhs)
//     }
//     pub fn or<Rhs>(self, rhs: Rhs) -> Or<Self, Rhs> {
//         or(self, rhs)
//     }
//     pub fn xor<Rhs>(self, rhs: Rhs) -> Xor<Self, Rhs> {
//         xor(self, rhs)
//     }
//     pub fn not(self) -> Not<Self> {
//         not(self)
//     }
// }

// /// ```
// /// use compacts::bits;
// /// let a = vec![0b_00001111_u8, 0b_10101010_u8];
// /// let b = vec![0b_11110000_u8, 0b_01010101_u8];
// /// let r = bits::and(a, b).into_iter().collect::<Vec<_>>();
// /// assert_eq!(r, vec![0, 0]);
// /// ```
pub fn and<L, R>(lhs: L, rhs: R) -> And<L, R> {
    Mask::mask(lhs, rhs)
}

// /// ```
// /// use compacts::bits;
// /// let a = vec![0b_00001111_u8, 0b_10101010_u8];
// /// let b = vec![0b_11110000_u8, 0b_01010101_u8];
// /// let r = bits::or(a, b).into_iter().collect::<Vec<_>>();
// /// assert_eq!(r, vec![!0, !0]);
// /// ```
pub fn or<L, R>(lhs: L, rhs: R) -> Or<L, R> {
    Mask::mask(lhs, rhs)
}

// /// ```
// /// use compacts::bits;
// /// let a = vec![0b_11001100_u8, 0b_11110000_u8];
// /// let b = vec![0b_11110000_u8, 0b_01010101_u8];
// /// let r = bits::xor(a, b).into_iter().collect::<Vec<_>>();
// /// assert_eq!(r, vec![0b_00111100, 0b_10100101]);
// /// ```
pub fn xor<L, R>(lhs: L, rhs: R) -> Xor<L, R> {
    Mask::mask(lhs, rhs)
}

// /// ```
// /// use compacts::bits;
// /// let a = bits::Map::<bits::Block<u64>>::new();
// /// let b = bits::Map::<bits::Block<u64>>::new();
// /// let r = a.and(!&b).into_iter().collect::<Vec<_>>();
// /// assert_eq!(r, vec![]);
// /// ```
// pub fn not<I>(data: I) -> Not<I> {
//     Not { data }
// }

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

impl<'a, L, R, A> Iterator for Iter<L, R, Cow<'a, bit::Block<A>>, sealed::And>
where
    L: Iterator<Item = Cow<'a, bit::Block<A>>>,
    R: Iterator<Item = Cow<'a, bit::Block<A>>>,
    A: bit::BlockArray,
{
    type Item = Cow<'a, bit::Block<A>>;
    fn next(&mut self) -> Option<Self::Item> {
        let lhs = &mut self.lhs;
        let rhs = &mut self.rhs;
        lhs.next().and_then(|mut x| {
            rhs.next().map(|y| {
                x.to_mut().bitand_assign(y.as_ref());
                x
            })
        })
    }
}

impl<'a, L, R, A> Iterator for Iter<L, R, Cow<'a, bit::Block<A>>, sealed::Or>
where
    L: Iterator<Item = Cow<'a, bit::Block<A>>>,
    R: Iterator<Item = Cow<'a, bit::Block<A>>>,
    A: bit::BlockArray,
{
    type Item = Cow<'a, bit::Block<A>>;
    fn next(&mut self) -> Option<Self::Item> {
        match (self.lhs.next(), self.rhs.next()) {
            (Some(mut x), Some(y)) => {
                x.to_mut().bitor_assign(y.as_ref());
                Some(x)
            }
            (Some(x), None) => Some(x),
            (None, Some(y)) => Some(y),
            (None, None) => None,
        }
    }
}

impl<'a, L, R, A> Iterator for Iter<L, R, Cow<'a, bit::Block<A>>, sealed::Xor>
where
    L: Iterator<Item = Cow<'a, bit::Block<A>>>,
    R: Iterator<Item = Cow<'a, bit::Block<A>>>,
    A: bit::BlockArray,
{
    type Item = Cow<'a, bit::Block<A>>;
    fn next(&mut self) -> Option<Self::Item> {
        match (self.lhs.next(), self.rhs.next()) {
            (Some(mut x), Some(y)) => {
                x.to_mut().bitxor_assign(y.as_ref());
                Some(x)
            }
            (Some(x), None) => Some(x),
            (None, Some(y)) => Some(y),
            (None, None) => None,
        }
    }
}

impl<'a, L, R, K, A> Iterator for Iter<L, R, bit::Entry<K, Cow<'a, bit::Block<A>>>, sealed::And>
where
    L: Iterator<Item = bit::Entry<K, Cow<'a, bit::Block<A>>>>,
    R: Iterator<Item = bit::Entry<K, Cow<'a, bit::Block<A>>>>,
    K: bit::Uint,
    A: bit::BlockArray,
{
    type Item = bit::Entry<K, Cow<'a, bit::Block<A>>>;
    fn next(&mut self) -> Option<Self::Item> {
        let lhs = &mut self.lhs;
        let rhs = &mut self.rhs;

        loop {
            match lhs
                .peek()
                .and_then(|x| rhs.peek().map(|y| x.index.cmp(&y.index)))
            {
                Some(Ordering::Equal) => {
                    let mut lhs = lhs.next().expect("peek");
                    let rhs = rhs.next().expect("peek");
                    lhs.value.to_mut().bitand_assign(rhs.value.as_ref());
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

impl<'a, L, R, K, A> Iterator for Iter<L, R, bit::Entry<K, Cow<'a, bit::Block<A>>>, sealed::Or>
where
    L: Iterator<Item = bit::Entry<K, Cow<'a, bit::Block<A>>>>,
    R: Iterator<Item = bit::Entry<K, Cow<'a, bit::Block<A>>>>,
    K: bit::Uint,
    A: bit::BlockArray,
{
    type Item = bit::Entry<K, Cow<'a, bit::Block<A>>>;
    fn next(&mut self) -> Option<Self::Item> {
        let lhs = &mut self.lhs;
        let rhs = &mut self.rhs;

        match (lhs.peek(), rhs.peek()) {
            (Some(l), Some(r)) => match l.index.cmp(&r.index) {
                Ordering::Less => lhs.next(),
                Ordering::Equal => {
                    let mut lhs = lhs.next().expect("peek");
                    let rhs = rhs.next().expect("peek");
                    lhs.value.to_mut().bitor_assign(rhs.value.as_ref());
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

impl<'a, L, R, K, A> Iterator for Iter<L, R, bit::Entry<K, Cow<'a, bit::Block<A>>>, sealed::Xor>
where
    L: Iterator<Item = bit::Entry<K, Cow<'a, bit::Block<A>>>>,
    R: Iterator<Item = bit::Entry<K, Cow<'a, bit::Block<A>>>>,
    K: bit::Uint,
    A: bit::BlockArray,
{
    type Item = bit::Entry<K, Cow<'a, bit::Block<A>>>;
    fn next(&mut self) -> Option<Self::Item> {
        let lhs = &mut self.lhs;
        let rhs = &mut self.rhs;

        match (lhs.peek(), rhs.peek()) {
            (Some(l), Some(r)) => match l.index.cmp(&r.index) {
                Ordering::Less => lhs.next(),
                Ordering::Equal => {
                    let mut lhs = lhs.next().expect("peek");
                    let rhs = rhs.next().expect("peek");
                    lhs.value.to_mut().bitxor_assign(rhs.value.as_ref());
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

// impl<'a, L, R, U> Iterator for Iter<L, R, Cow<'a, [U]>, sealed::And>
// where
//     L: Iterator<Item = Cow<'a, [U]>>,
//     R: Iterator<Item = Cow<'a, [U]>>,
//     U: Uint,
// {
//     type Item = Cow<'a, [U]>;
//     fn next(&mut self) -> Option<Self::Item> {
//         let lhs = &mut self.lhs;
//         let rhs = &mut self.rhs;
//         lhs.next().and_then(|mut x| {
//             rhs.next().map(|y| {
//                 for (a, b) in x.to_mut().iter_mut().zip(y.iter()) {
//                     *a &= *b;
//                 }
//                 x
//             })
//         })
//     }
// }

// impl<'a, L, R, U> Iterator for Iter<L, R, Cow<'a, [U]>, sealed::Or>
// where
//     L: Iterator<Item = Cow<'a, [U]>>,
//     R: Iterator<Item = Cow<'a, [U]>>,
//     U: Uint,
// {
//     type Item = Cow<'a, [U]>;
//     fn next(&mut self) -> Option<Self::Item> {
//         match (self.lhs.next(), self.rhs.next()) {
//             (Some(mut x), Some(y)) => {
//                 for (a, b) in x.to_mut().iter_mut().zip(y.iter()) {
//                     *a |= *b;
//                 }
//                 Some(x)
//             }
//             (Some(x), None) => Some(x),
//             (None, Some(y)) => Some(y),
//             (None, None) => None,
//         }
//     }
// }

// impl<'a, L, R, U> Iterator for Iter<L, R, Cow<'a, [U]>, sealed::Xor>
// where
//     L: Iterator<Item = Cow<'a, [U]>>,
//     R: Iterator<Item = Cow<'a, [U]>>,
//     U: Uint,
// {
//     type Item = Cow<'a, [U]>;
//     fn next(&mut self) -> Option<Self::Item> {
//         match (self.lhs.next(), self.rhs.next()) {
//             (Some(mut x), Some(y)) => {
//                 for (a, b) in x.to_mut().iter_mut().zip(y.iter()) {
//                     *a ^= *b;
//                 }
//                 Some(x)
//             }
//             (Some(x), None) => Some(x),
//             (None, Some(y)) => Some(y),
//             (None, None) => None,
//         }
//     }
// }

// impl<'a, L, R, V> Iterator for Iter<L, R, Cow<'a, V>, sealed::And>
// where
//     L: Iterator<Item = Cow<'a, V>>,
//     R: Iterator<Item = Cow<'a, V>>,
//     V: BitAndAssign<Cow<'a, V>> + Clone + 'a,
// {
//     type Item = Cow<'a, V>;
//     fn next(&mut self) -> Option<Self::Item> {
//         let lhs = &mut self.lhs;
//         let rhs = &mut self.rhs;
//         lhs.next().and_then(|mut x| {
//             rhs.next().map(|y| {
//                 x.to_mut().bitand_assign(y);
//                 x
//             })
//         })
//     }
// }

// impl<'a, L, R, V> Iterator for Iter<L, R, Cow<'a, V>, sealed::Or>
// where
//     L: Iterator<Item = Cow<'a, V>>,
//     R: Iterator<Item = Cow<'a, V>>,
//     V: BitOrAssign<Cow<'a, V>> + Clone + 'a,
// {
//     type Item = Cow<'a, V>;
//     fn next(&mut self) -> Option<Self::Item> {
//         match (self.lhs.next(), self.rhs.next()) {
//             (Some(mut x), Some(y)) => {
//                 x.to_mut().bitor_assign(y);
//                 Some(x)
//             }
//             (Some(x), None) => Some(x),
//             (None, Some(y)) => Some(y),
//             (None, None) => None,
//         }
//     }
// }

// impl<'a, L, R, V> Iterator for Iter<L, R, Cow<'a, V>, sealed::Xor>
// where
//     L: Iterator<Item = Cow<'a, V>>,
//     R: Iterator<Item = Cow<'a, V>>,
//     V: BitXorAssign<Cow<'a, V>> + Clone + 'a,
// {
//     type Item = Cow<'a, V>;
//     fn next(&mut self) -> Option<Self::Item> {
//         match (self.lhs.next(), self.rhs.next()) {
//             (Some(mut x), Some(y)) => {
//                 x.to_mut().bitxor_assign(y);
//                 Some(x)
//             }
//             (Some(x), None) => Some(x),
//             (None, Some(y)) => Some(y),
//             (None, None) => None,
//         }
//     }
// }

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
    /// use compacts::bit::{VecMap, Fold, ops::Access};
    /// let a = VecMap::<[u64; 1024]>::build(&[1, 2, 4, 5, 10]);
    /// let b = VecMap::<[u64; 1024]>::build(&[1, 3, 4, 8, 10]);
    /// let c = VecMap::<[u64; 1024]>::build(&[1, 2, 4, 9, 10]);
    /// let fold = Fold::and(vec![&a, &b, &c]).collect::<VecMap<[u64; 1024]>>();
    /// let bits = fold.iterate().collect::<Vec<_>>();
    /// assert_eq!(bits, vec![1, 4, 10]);
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

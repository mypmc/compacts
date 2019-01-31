use std::{
    borrow::Cow,
    iter::Peekable,
    marker::PhantomData,
    ops::{Not, RangeFrom},
};

use crate::bits::*;

pub struct Flip<T> {
    pub(crate) bits: u64,
    pub(crate) data: T,
}

impl<T, B> IntoIterator for Flip<T>
where
    T: IntoIterator<Item = B>,
    Iter<T::IntoIter, B>: Iterator<Item = B>,
{
    type Item = B;
    type IntoIter = Iter<T::IntoIter, B>;
    fn into_iter(self) -> Self::IntoIter {
        let pad = PadDefault::pad_default(self.data.into_iter(), self.bits);
        Iter { pad }
    }
}

pub struct Iter<I: Iterator, B> {
    pad: PadDefault<I, B>,
}

impl<'a, I, V> Iterator for Iter<I, V>
where
    I: Iterator<Item = V>,
    V: UnsignedInt,
{
    type Item = V;
    fn next(&mut self) -> Option<Self::Item> {
        self.pad.next().map(|n| !n)
    }
}

impl<'a, I, V> Iterator for Iter<I, Cow<'a, V>>
where
    I: Iterator<Item = Cow<'a, V>>,
    V: FiniteBits + Not<Output = V> + 'a,
{
    type Item = Cow<'a, V>;
    fn next(&mut self) -> Option<Self::Item> {
        self.pad.next().map(|cow| Cow::Owned(!cow.into_owned()))
    }
}

impl<'a, I, K, V> Iterator for Iter<I, Entry<K, Cow<'a, V>>>
where
    I: Iterator<Item = Entry<K, Cow<'a, V>>>,
    K: UnsignedInt,
    V: FiniteBits + Not<Output = V> + 'a,
{
    type Item = Entry<K, Cow<'a, V>>;
    fn next(&mut self) -> Option<Self::Item> {
        self.pad.next().map(|page| {
            let index = page.index;
            let owned = page.value.into_owned();
            let value = Cow::Owned(!owned);
            Entry::new(index, value)
        })
    }
}

pub struct PadDefault<I: Iterator, B> {
    value: Peekable<I>,
    dummy: RangeFrom<u64>,
    bits: u64,
    item: PhantomData<B>,
}

impl<I: Iterator, B> PadDefault<I, B> {
    fn pad_default(iter: I, bits: u64) -> Self {
        PadDefault {
            value: iter.peekable(),
            dummy: 0..,
            bits,
            item: PhantomData,
        }
    }
}

enum PadItem<K> {
    Dummy(K),
    Found,
    Empty,
}

impl<I, V> Iterator for PadDefault<I, V>
where
    V: UnsignedInt,
    I: Iterator<Item = V>,
{
    type Item = V;

    fn next(&mut self) -> Option<Self::Item> {
        match (self.dummy.next(), self.value.peek()) {
            (Some(_), Some(_)) => self.value.next(),
            (Some(i), None) => {
                if i < self.bits / V::BITS {
                    Some(V::empty())
                } else {
                    None
                }
            }
            (None, Some(_)) => unreachable!("should not happen"),
            (None, None) => None,
        }
    }
}

impl<'a, I, V> Iterator for PadDefault<I, Cow<'a, V>>
where
    V: FiniteBits,
    I: Iterator<Item = Cow<'a, V>>,
{
    type Item = Cow<'a, V>;

    fn next(&mut self) -> Option<Self::Item> {
        match (self.dummy.next(), self.value.peek()) {
            (Some(_), Some(_)) => self.value.next(),
            (Some(i), None) => {
                if i < self.bits / V::BITS {
                    Some(Cow::Owned(V::empty()))
                } else {
                    None
                }
            }
            (None, Some(_)) => unreachable!("should not happen"),
            (None, None) => None,
        }
    }
}

impl<'a, I, K, V> Iterator for PadDefault<I, Entry<K, Cow<'a, V>>>
where
    K: UnsignedInt,
    V: FiniteBits,
    I: Iterator<Item = Entry<K, Cow<'a, V>>>,
{
    type Item = Entry<K, Cow<'a, V>>;

    fn next(&mut self) -> Option<Self::Item> {
        use PadItem::*;
        let item = match (self.dummy.next(), self.value.peek()) {
            (Some(i), Some(p)) => {
                let j = ucast::<K, u64>(p.index);
                if i < j {
                    Dummy(ucast::<u64, K>(i))
                } else if i == j {
                    Found
                } else {
                    unreachable!("dummy index > entry")
                }
            }
            (Some(i), None) => {
                if i < self.bits / V::BITS {
                    Dummy(ucast::<u64, K>(i))
                } else {
                    Empty
                }
            }
            // (None, Some(_)) => Found,
            (None, Some(_)) => unreachable!("should not happen"),
            (None, None) => Empty,
        };

        match item {
            Dummy(k) => Some(Entry::new(k, Cow::Owned(V::empty()))),
            Found => self.value.next(),
            Empty => None,
        }
    }
}

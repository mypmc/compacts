// Iterators for Bucket.
// Iter is borrowing iterator, IntoIter is consuming iterator.

use std::iter::{Iterator, FromIterator, IntoIterator, Extend, ExactSizeIterator};
use std::slice::Iter as SliceIter;
use std::vec::IntoIter as VecIntoIter;
use std::marker::PhantomData;
use std::borrow::Cow;

use {bits, dir, Bucket, PopCount};

impl Bucket {
    pub fn iter<'a>(&'a self) -> Iter<'a> {
        match self {
            &Bucket::Vec(ref pop, ref bits) => {
                debug_assert!(pop.value() == bits.len() as u64);
                debug_assert!(pop.value() <= Bucket::CAPACITY);
                let iter = bits.iter();
                Iter::Vec(iter)
            }

            &Bucket::Map(ref pop, ref bits) => {
                debug_assert!(pop.value() <= Bucket::CAPACITY);
                let ptr = Pointer::new(Cow::Borrowed(&bits[..]));
                Iter::Map { pop, ptr }
            }
        }
    }
    pub fn into_iter(self) -> IntoIter {
        match self {
            Bucket::Vec(pop, bits) => {
                debug_assert!(pop.value() == bits.len() as u64);
                debug_assert!(pop.value() <= Bucket::CAPACITY);
                let iter = bits.into_iter();
                IntoIter::Vec(iter)
            }
            Bucket::Map(pop, box bits) => {
                debug_assert!(pop.value() <= Bucket::CAPACITY);
                let ptr = Pointer::new(Cow::Owned(bits.to_vec()));
                IntoIter::Map { pop, ptr }
            }
        }
    }
}

#[derive(Debug)]
pub enum Iter<'a> {
    Vec(SliceIter<'a, u16>),

    Map {
        pop: &'a bits::Count<u16>,
        ptr: Pointer<'a, dir::Forward>,
    },
}
#[derive(Debug)]
pub enum IntoIter {
    Vec(VecIntoIter<u16>),

    Map {
        pop: bits::Count<u16>,
        ptr: Pointer<'static, dir::Forward>,
    },
}

#[derive(Debug)]
pub struct Pointer<'a, T: dir::Direction> {
    idx: usize,
    pos: usize,
    cow: Cow<'a, [u64]>,
    dir: PhantomData<T>,
}

impl<'a> Pointer<'a, dir::Forward> {
    fn new(cow: Cow<'a, [u64]>) -> Self {
        let idx = 0;
        let pos = 0;
        Pointer {
            idx,
            pos,
            cow,
            dir: PhantomData,
        }
    }

    fn goto_next(&mut self) {
        self.pos += 1;
        if self.pos == Bucket::BITS_SIZE {
            self.pos = 0;
            self.idx += 1;
        }
    }
}

impl<'a> Pointer<'a, dir::Forward> {
    fn next(&mut self) -> Option<u16> {
        loop {
            let i = self.idx;
            let p = self.pos;
            if i >= self.cow.len() {
                return None;
            } else if self.cow[i] & (1u64 << p) != 0 {
                let bit = Some((i * Bucket::BITS_SIZE + p) as u16);
                self.goto_next();
                return bit;
            }
            self.goto_next();
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = u16;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            &mut Iter::Vec(ref mut it) => it.next().cloned(),
            &mut Iter::Map { ref mut ptr, .. } => ptr.next(),
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            &Iter::Vec(ref it) => it.size_hint(),
            &Iter::Map { ref pop, .. } => {
                let ones = pop.value() as usize;
                (ones, Some(ones))
            }
        }
    }
}
impl<'a> ExactSizeIterator for Iter<'a> {}

impl Iterator for IntoIter {
    type Item = u16;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            &mut IntoIter::Vec(ref mut it) => it.next(),
            &mut IntoIter::Map { ref mut ptr, .. } => ptr.next(),
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            &IntoIter::Vec(ref it) => it.size_hint(),
            &IntoIter::Map { ref pop, .. } => {
                let ones = pop.value() as usize;
                (ones, Some(ones))
            }
        }
    }
}
impl ExactSizeIterator for IntoIter {}

impl FromIterator<u16> for Bucket {
    fn from_iter<I: IntoIterator<Item = u16>>(iterable: I) -> Self {
        let iter = iterable.into_iter();
        let (min, maybe) = iter.size_hint();
        let mut bucket = Bucket::with_capacity(if let Some(max) = maybe { max } else { min });
        let ones = extend_by_u16(&mut bucket, iter);
        debug_assert_eq!(ones, bucket.ones());
        bucket
    }
}
impl<'a> FromIterator<&'a u16> for Bucket {
    fn from_iter<I: IntoIterator<Item = &'a u16>>(iterable: I) -> Self {
        let iter = iterable.into_iter();
        iter.cloned().collect::<Bucket>()
    }
}
impl FromIterator<bool> for Bucket {
    fn from_iter<I: IntoIterator<Item = bool>>(iterable: I) -> Bucket {
        let iter = iterable.into_iter();
        iter.take(Bucket::CAPACITY as usize)
            .enumerate()
            .filter_map(|(i, p)| if p { Some(i as u16) } else { None })
            .collect::<Bucket>()
    }
}
impl<'a> FromIterator<&'a bool> for Bucket {
    fn from_iter<I: IntoIterator<Item = &'a bool>>(iterable: I) -> Bucket {
        let iter = iterable.into_iter();
        iter.cloned().collect::<Bucket>()
    }
}

fn extend_by_u16<I: Iterator<Item = u16>>(bucket: &mut Bucket, iter: I) -> u64 {
    let mut ones = 0;
    for item in iter {
        if bucket.insert(item) {
            ones += 1;
        }
    }
    ones
}

impl<'a> IntoIterator for &'a Bucket {
    type Item = <Iter<'a> as Iterator>::Item;
    type IntoIter = Iter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl IntoIterator for Bucket {
    type Item = <IntoIter as Iterator>::Item;
    type IntoIter = IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.into_iter()
    }
}

impl Extend<u16> for Bucket {
    fn extend<I: IntoIterator<Item = u16>>(&mut self, iterable: I) {
        extend_by_u16(self, iterable.into_iter());
    }
}

use std::iter::{Iterator, ExactSizeIterator};
use std::slice::Iter as SliceIter;
use std::marker::PhantomData;

use {bits, PopCount};
use Bucket;
use {Direction, Forward};

/// module document.
// each 'ones' are count of non-zero bit; for size_hint
#[derive(Debug, Clone)]
pub enum Iter<'a> {
    Vec {
        pop: &'a bits::Count<u16>,
        iter: SliceIter<'a, u16>,
    },
    Map {
        pop: &'a bits::Count<u16>,
        ptr: SlicePtr<'a, Forward>,
    },
}

impl<'a> Iter<'a> {
    pub fn vec(bits: &'a [u16], pop: &'a bits::Count<u16>) -> Iter<'a> {
        debug_assert!(pop.value() == bits.len() as u64);
        debug_assert!(pop.value() <= Bucket::CAPACITY);
        let iter = bits.iter();
        Iter::Vec { pop, iter }
    }
    pub fn map(bits: &'a [u64], pop: &'a bits::Count<u16>) -> Iter<'a> {
        debug_assert!(pop.value() <= Bucket::CAPACITY);
        let ptr = SlicePtr::<'a, Forward>::new(bits);
        Iter::Map { pop, ptr }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = u16;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            &mut Iter::Vec { ref mut iter, .. } => iter.next().cloned(),
            &mut Iter::Map { ref mut ptr, .. } => ptr.next(),
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            &Iter::Vec { ref iter, .. } => iter.size_hint(),
            &Iter::Map { ref pop, .. } => {
                let ones = pop.value() as usize;
                (ones, Some(ones))
            }
        }
    }
}
impl<'a> ExactSizeIterator for Iter<'a> {
    /*
    fn is_empty(&self) -> bool {
        match self {
            &Iter::Vec { ref ones, .. } => ones == 0,
            &Iter::Map { ref ones, .. } => ones == 0,
        }
    }
    */
}

#[derive(Debug, Clone)]
pub struct SlicePtr<'a, T: Direction> {
    bits: &'a [u64],
    idx: usize,
    pos: usize,
    _dir: PhantomData<T>,
}

impl<'a> SlicePtr<'a, Forward> {
    fn new(bits: &'a [u64]) -> Self {
        SlicePtr {
            bits,
            idx: 0,
            pos: 0,
            _dir: PhantomData,
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

impl<'a> SlicePtr<'a, Forward> {
    fn next(&mut self) -> Option<u16> {
        loop {
            let i = self.idx;
            let p = self.pos;
            if i >= self.bits.len() {
                return None;
            } else if self.bits[i] & (1u64 << p) != 0 {
                let bit = Some((i * Bucket::BITS_SIZE + p) as u16);
                self.goto_next();
                return bit;
            }
            self.goto_next();
        }
    }
}

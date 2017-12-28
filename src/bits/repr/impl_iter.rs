use std::iter::IntoIterator;
use super::{ArrBlock, RunBlock, SeqBlock};
use super::{BoxedIter, OwnedIter};

impl<'a> Iterator for BoxedIter<'a> {
    type Item = u16;
    fn next(&mut self) -> Option<Self::Item> {
        if let some @ Some(_) = self.iter.next() {
            self.rest -= 1;
            some
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.rest as usize;
        (len, Some(len))
    }
}

impl<'a> ExactSizeIterator for BoxedIter<'a> {
    fn len(&self) -> usize {
        self.rest as usize
    }
}

impl Iterator for OwnedIter {
    type Item = u16;
    fn next(&mut self) -> Option<Self::Item> {
        if let some @ Some(_) = self.iter.next() {
            self.rest -= 1;
            some
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.rest as usize;
        (len, Some(len))
    }
}

impl ExactSizeIterator for OwnedIter {
    fn len(&self) -> usize {
        self.rest as usize
    }
}

struct Word {
    i: usize, // bitmap index
    p: usize, // shift position
    word: u64,
}

impl Word {
    fn new(i: usize, word: u64) -> Self {
        let p = 0;
        Word { i, p, word }
    }
}

impl From<(usize, u64)> for Word {
    fn from(d: (usize, u64)) -> Self {
        Word::new(d.0, d.1)
    }
}

impl Iterator for Word {
    type Item = u16;
    fn next(&mut self) -> Option<Self::Item> {
        let &mut Word { word, i, ref mut p } = self;
        loop {
            if *p >= 64 {
                break None;
            }
            if word & (1u64 << *p) != 0 {
                let bit = (i * 64 + (*p)) as u16;
                *p += 1;
                break Some(bit);
            }
            *p += 1;
        }
    }
}

impl<'a> IntoIterator for &'a SeqBlock {
    type Item = u16;
    type IntoIter = BoxedIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        let rest = self.vector.len() as u32;
        let iter = Box::new(self.vector.iter().cloned());
        BoxedIter { rest, iter }
    }
}

impl IntoIterator for SeqBlock {
    type Item = u16;
    type IntoIter = OwnedIter;
    fn into_iter(self) -> Self::IntoIter {
        let rest = self.vector.len() as u32;
        let iter = Box::new(self.vector.into_iter());
        OwnedIter { rest, iter }
    }
}

impl<'a> IntoIterator for &'a ArrBlock {
    type Item = u16;
    type IntoIter = BoxedIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        let rest = self.weight;
        let iter = self.bitmap
            .iter()
            .enumerate()
            .flat_map(|(index, &packed)| Word::new(index, packed));
        let iter = Box::new(iter);
        BoxedIter { rest, iter }
    }
}

impl IntoIterator for ArrBlock {
    type Item = u16;
    type IntoIter = OwnedIter;
    fn into_iter(self) -> Self::IntoIter {
        let rest = self.weight;
        let iter = self.bitmap
            .to_vec()
            .into_iter()
            .enumerate()
            .flat_map(|(index, packed)| Word::new(index, packed));
        let iter = Box::new(iter);
        OwnedIter { rest, iter }
    }
}

impl<'a> IntoIterator for &'a RunBlock {
    type Item = u16;
    type IntoIter = BoxedIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        let rest = self.weight;
        let iter = (&self.ranges)
            .iter()
            .flat_map(|range| range.start..=range.end);
        let iter = Box::new(iter);
        BoxedIter { rest, iter }
    }
}

impl IntoIterator for RunBlock {
    type Item = u16;
    type IntoIter = OwnedIter;
    fn into_iter(self) -> Self::IntoIter {
        let rest = self.weight;
        let iter = self.ranges
            .into_iter()
            .flat_map(|range| range.start..=range.end);
        let iter = Box::new(iter);
        OwnedIter { rest, iter }
    }
}

use std::iter::IntoIterator;

pub(crate) use super::Seq16;
pub(crate) use super::Arr64;
pub(crate) use super::Run16;

pub(crate) struct Boxed<'a> {
    rest: u32,
    iter: Box<Iterator<Item = u16> + 'a>,
}

pub(crate) struct Owned {
    rest: u32,
    iter: Box<Iterator<Item = u16>>,
}

pub(crate) struct Packed {
    shift: usize,
    entry: (usize, u64),
}

impl Packed {
    fn new(index: usize, packed: u64) -> Self {
        let shift = 0;
        let entry = (index, packed);
        Packed { shift, entry }
    }
}

impl From<(usize, u64)> for Packed {
    fn from(d: (usize, u64)) -> Self {
        Packed::new(d.0, d.1)
    }
}

impl Iterator for Packed {
    type Item = u16;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.shift >= 64 {
                break None;
            }
            let (index, packed) = self.entry;
            if packed & (1u64 << self.shift) != 0 {
                let bit = (index * 64 + self.shift) as u16;
                self.shift += 1;
                break Some(bit);
            }
            self.shift += 1;
        }
    }
}

impl<'a> IntoIterator for &'a Seq16 {
    type Item = u16;
    type IntoIter = Boxed<'a>;
    fn into_iter(self) -> Self::IntoIter {
        let rest = self.vector.len() as u32;
        let iter = Box::new((&self.vector).iter().cloned());
        Boxed { rest, iter }
    }
}
impl IntoIterator for Seq16 {
    type Item = u16;
    type IntoIter = Owned;
    fn into_iter(self) -> Self::IntoIter {
        let rest = self.vector.len() as u32;
        let iter = Box::new(self.vector.into_iter());
        Owned { rest, iter }
    }
}

impl<'a> IntoIterator for &'a Arr64 {
    type Item = u16;
    type IntoIter = Boxed<'a>;
    fn into_iter(self) -> Self::IntoIter {
        let rest = self.weight;
        let iter = self.boxarr
            .iter()
            .enumerate()
            .flat_map(|(index, &packed)| Packed::new(index, packed));
        let iter = Box::new(iter);
        Boxed { rest, iter }
    }
}
impl IntoIterator for Arr64 {
    type Item = u16;
    type IntoIter = Owned;
    fn into_iter(self) -> Self::IntoIter {
        let rest = self.weight;
        let iter = (self.boxarr as Box<[_]>)
            .into_vec()
            .into_iter()
            .enumerate()
            .flat_map(|(index, packed)| Packed::new(index, packed));
        let iter = Box::new(iter);
        Owned { rest, iter }
    }
}

impl<'a> IntoIterator for &'a Run16 {
    type Item = u16;
    type IntoIter = Boxed<'a>;
    fn into_iter(self) -> Self::IntoIter {
        let rest = self.weight;
        let iter = (&self.ranges)
            .iter()
            .flat_map(|range| range.start..=range.end);
        let iter = Box::new(iter);
        Boxed { rest, iter }
    }
}
impl IntoIterator for Run16 {
    type Item = u16;
    type IntoIter = Owned;
    fn into_iter(self) -> Self::IntoIter {
        let rest = self.weight;
        let iter = self.ranges
            .into_iter()
            .flat_map(|range| range.start..=range.end);
        let iter = Box::new(iter);
        Owned { rest, iter }
    }
}

impl<'a> Iterator for Boxed<'a> {
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

impl<'a> ExactSizeIterator for Boxed<'a> {
    fn len(&self) -> usize {
        self.rest as usize
    }
}

impl Iterator for Owned {
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

impl ExactSizeIterator for Owned {
    fn len(&self) -> usize {
        self.rest as usize
    }
}

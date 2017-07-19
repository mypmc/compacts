use std::ops::RangeInclusive;
use std::mem;

use pair::*;
use super::{Block, Rle16, Seq16, Seq64};
use super::range::{self, TwoFold};

pub struct Rle16Iter<'a> {
    boxed: Box<Iterator<Item = u16> + 'a>,
    len: usize,
}

impl Rle16 {
    pub fn new() -> Self {
        Rle16::default()
    }

    pub fn iter(&self) -> Rle16Iter {
        let len = self.weight as usize;
        let boxed = Box::new(
            self.ranges
                .iter()
                .flat_map(|range| (range.start...range.end).into_iter()),
        );
        Rle16Iter { boxed, len }
    }

    pub fn count_ones(&self) -> u32 {
        self.weight
    }

    pub fn count_zeros(&self) -> u32 {
        Block::CAPACITY as u32 - self.count_ones()
    }

    pub fn count_rle(&self) -> usize {
        self.ranges.len()
    }

    pub fn size(run_length: usize) -> usize {
        run_length * mem::size_of::<RangeInclusive<u16>>() + mem::size_of::<u32>()
    }

    pub fn mem_size(&self) -> usize {
        Self::size(self.ranges.len())
    }

    pub fn search(&self, x: &u16) -> Result<usize, usize> {
        use std::cmp::Ordering;
        self.ranges.binary_search_by(
            |range| if range.start <= *x && *x <= range.end {
                Ordering::Equal
            } else if *x < range.start {
                Ordering::Greater
            } else if range.end < *x {
                Ordering::Less
            } else {
                unreachable!()
            },
        )
    }

    pub fn index_to_insert(&self, x: &u16) -> Option<usize> {
        self.search(x).err()
    }

    pub fn index_to_remove(&self, x: &u16) -> Option<usize> {
        self.search(x).ok()
    }

    pub fn contains(&self, x: u16) -> bool {
        self.search(&x).is_ok()
    }

    pub fn insert(&mut self, x: u16) -> bool {
        if let Some(pos) = self.index_to_insert(&x) {
            self.weight += 1;

            let lhs = if pos > 0 && pos <= self.ranges.len() {
                Some(self.ranges[pos - 1].end)
            } else {
                None
            };
            let rhs = if pos < (::std::u16::MAX as usize) && pos < self.ranges.len() {
                Some(self.ranges[pos].start)
            } else {
                None
            };

            match (lhs, rhs) {
                (None, Some(rhs)) if x == rhs - 1 => {
                    self.ranges[pos] = (self.ranges[pos].start - 1)...self.ranges[pos].end;
                }
                (Some(lhs), Some(rhs)) if lhs + 1 == x && x == rhs - 1 => {
                    let i = pos - 1;
                    self.ranges[i] = self.ranges[i].start...self.ranges[pos].end;
                    self.ranges.remove(pos);
                }
                (Some(lhs), _) if lhs + 1 == x => {
                    let i = pos - 1;
                    self.ranges[i] = self.ranges[i].start...(self.ranges[i].end + 1);
                }
                (_, Some(rhs)) if x == rhs - 1 => {
                    self.ranges[pos] = (self.ranges[pos].start - 1)...self.ranges[pos].end;
                }
                _ => {
                    self.ranges.insert(pos, x...x);
                }
            }
            true
        } else {
            false
        }
    }

    pub fn remove(&mut self, x: u16) -> bool {
        if let Some(pos) = self.index_to_remove(&x) {
            self.weight -= 1;

            match (self.ranges[pos].start, self.ranges[pos].end) {
                (i, j) if i == j => {
                    self.ranges.remove(pos);
                }
                (i, j) if i < x && x < j => {
                    self.ranges.remove(pos);
                    self.ranges.insert(pos, i...(x - 1));
                    self.ranges.insert(pos + 1, (x + 1)...j);
                }
                (i, j) if i == x => {
                    assert!(i + 1 <= j);
                    self.ranges[pos] = (i + 1)...j;
                }
                (i, j) if j == x => {
                    assert!(i <= j - 1);
                    self.ranges[pos] = i...(j - 1);
                }
                _ => unreachable!(),
            };
            true
        } else {
            false
        }
    }
}

impl From<Seq16> for Rle16 {
    fn from(vec16: Seq16) -> Self {
        Rle16::from(&vec16)
    }
}
impl<'a> From<&'a Seq16> for Rle16 {
    fn from(vec16: &'a Seq16) -> Self {
        vec16.vector.iter().collect()
    }
}

impl From<Seq64> for Rle16 {
    fn from(vec64: Seq64) -> Self {
        Rle16::from(&vec64)
    }
}
impl<'a> From<&'a Seq64> for Rle16 {
    fn from(vec64: &'a Seq64) -> Self {
        const WIDTH: u16 = <u64 as ::UnsignedInt>::WIDTH as u16;
        let mut rle = Rle16::new();
        let enumerate = vec64.vector.iter().enumerate();
        for (i, &bit) in enumerate.filter(|&(_, &v)| v != 0) {
            let mut word = bit;
            for pos in 0..WIDTH {
                if word & (1 << pos) != 0 {
                    let x = (i as u16 * WIDTH) + pos;
                    rle.insert(x);
                    word &= !(1 << pos);
                }
            }
        }
        rle
    }
}

impl<'a> ::std::iter::FromIterator<u16> for Rle16 {
    fn from_iter<I>(iterable: I) -> Self
    where
        I: IntoIterator<Item = u16>,
    {
        let mut rle = Rle16::new();
        for bit in iterable {
            rle.insert(bit);
        }
        rle
    }
}
impl<'a> ::std::iter::FromIterator<&'a u16> for Rle16 {
    fn from_iter<I>(iterable: I) -> Self
    where
        I: IntoIterator<Item = &'a u16>,
    {
        iterable.into_iter().cloned().collect()
    }
}

impl<'a> From<&'a [RangeInclusive<u16>]> for Rle16 {
    fn from(slice: &'a [RangeInclusive<u16>]) -> Self {
        let mut rle16 = Rle16 {
            weight: 0,
            ranges: Vec::with_capacity(slice.len()),
        };
        for r in slice {
            let w = (r.end - r.start) as u32 + 1;
            rle16.weight += w;
            rle16.ranges.push(r.start...r.end);
        }
        rle16
    }
}

macro_rules! impl_Pairwise {
    ( $( ( $op:ident, $fn:ident ) ),* ) => ($(
        impl<'a, 'b> $op<&'b Rle16> for &'a Rle16 {
            type Output = Rle16;
            fn $fn(self, rle16: &'b Rle16) -> Self::Output {
                let fold = TwoFold::new(&self.ranges, &rle16.ranges).$fn();
                let (weight, ranges) = range::repair(fold);
                Rle16 { weight, ranges }
            }
        }
    )*)
}

impl_Pairwise!(
    (Intersection, intersection),
    (Union, union),
    (Difference, difference),
    (SymmetricDifference, symmetric_difference)
);

impl<'a> IntersectionWith<&'a Rle16> for Rle16 {
    fn intersection_with(&mut self, rle16: &'a Rle16) {
        *self = (&*self).intersection(rle16);
    }
}
impl<'a> UnionWith<&'a Rle16> for Rle16 {
    fn union_with(&mut self, rle16: &'a Rle16) {
        *self = (&*self).union(rle16);
    }
}
impl<'a> DifferenceWith<&'a Rle16> for Rle16 {
    fn difference_with(&mut self, rle16: &'a Rle16) {
        *self = (&*self).difference(rle16);
    }
}
impl<'a> SymmetricDifferenceWith<&'a Rle16> for Rle16 {
    fn symmetric_difference_with(&mut self, rle16: &'a Rle16) {
        *self = (&*self).symmetric_difference(rle16);
    }
}

impl<'a> Iterator for Rle16Iter<'a> {
    type Item = u16;
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.boxed.next();
        if next.is_some() {
            self.len -= 1;
        }
        next
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a> ExactSizeIterator for Rle16Iter<'a> {
    fn len(&self) -> usize {
        self.len
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! insert_all {
        ( $rle:expr $(, $x:expr )* ) => ($(
            assert!($rle.insert($x));
        )*)
    }

    macro_rules! remove_all {
        ( $rle:expr $(, $x:expr )* ) => ($(
            assert!($rle.remove($x));
        )*)
    }

    fn test_identity(rle: &Rle16) {
        let from_seq16 = Rle16::from(Seq16::from(rle));
        let from_seq64 = Rle16::from(Seq64::from(rle));
        assert_eq!(rle.weight, from_seq16.weight);
        assert_eq!(rle.ranges, from_seq16.ranges);
        assert_eq!(rle.weight, from_seq64.weight);
        assert_eq!(rle.ranges, from_seq64.ranges);
    }

    #[test]
    fn rle16_insert_remove() {
        let mut rle = Rle16::new();

        test_identity(&rle);
        insert_all!(rle, 1, 3, 5, 4);
        assert_eq!(rle.count_ones(), 4);
        assert_eq!(rle.ranges, &[1...1, 3...5]);

        test_identity(&rle);
        insert_all!(rle, 2, 8);
        assert_eq!(rle.count_ones(), 6);
        assert_eq!(rle.ranges, &[1...5, 8...8]);

        test_identity(&rle);
        insert_all!(rle, 10, 7);
        assert_eq!(rle.count_ones(), 8);
        assert_eq!(rle.ranges, &[1...5, 7...8, 10...10]);

        test_identity(&rle);
        insert_all!(rle, 9, 6, 0);
        assert_eq!(rle.count_ones(), 11);
        assert_eq!(rle.ranges, &[0...10]);

        test_identity(&rle);
        insert_all!(rle, 65534, 65535);
        assert_eq!(rle.count_ones(), 13);
        assert_eq!(rle.ranges, &[0...10, 65534...65535]);

        test_identity(&rle);
        remove_all!(rle, 65534, 65535);
        assert_eq!(rle.count_ones(), 11);
        assert_eq!(rle.ranges, &[0...10]);

        test_identity(&rle);
        remove_all!(rle, 0, 4);
        assert_eq!(rle.count_ones(), 9);
        assert_eq!(rle.ranges, &[1...3, 5...10]);

        test_identity(&rle);
        remove_all!(rle, 7, 2);
        assert_eq!(rle.count_ones(), 7);
        assert_eq!(rle.ranges, &[1...1, 3...3, 5...6, 8...10]);

        test_identity(&rle);
        remove_all!(rle, 3, 5);
        assert_eq!(rle.count_ones(), 5);
        assert_eq!(rle.ranges, &[1...1, 6...6, 8...10]);

        test_identity(&rle);
        remove_all!(rle, 1, 6);
        assert_eq!(rle.count_ones(), 3);
        assert_eq!(rle.ranges, &[8...10]);

        test_identity(&rle);
        remove_all!(rle, 10, 8);
        assert_eq!(rle.count_ones(), 1);
        assert_eq!(rle.ranges, &[9...9]);

        test_identity(&rle);
        remove_all!(rle, 9);
        assert_eq!(rle.count_ones(), 0);
        assert_eq!(rle.ranges, &[]);
    }
}

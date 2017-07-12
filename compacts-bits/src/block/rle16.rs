use std::ops::RangeInclusive;

use ops::*;
use super::{Seq16, Seq64, Rle16};
use super::range::{self, TwoFold};
use Rank;

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

impl Seq16 {
    pub fn count_rle(&self) -> usize {
        let rle = Rle16::from(self);
        rle.ranges.len()
    }
}

impl Seq64 {
    pub fn count_rle(&self) -> usize {
        let rle = Rle16::from(self);
        rle.ranges.len()
    }
}

impl Rle16 {
    pub fn new() -> Self {
        Rle16::default()
    }

    pub fn count_rle(&self) -> usize {
        self.ranges.len()
    }

    pub fn search(&self, x: u16) -> Result<usize, usize> {
        use std::cmp::Ordering;
        self.ranges.binary_search_by(
            |range| if range.start <= x && x <= range.end {
                Ordering::Equal
            } else if x < range.start {
                Ordering::Greater
            } else if range.end < x {
                Ordering::Less
            } else {
                unreachable!()
            },
        )
    }

    fn index_to_insert(&self, x: u16) -> Option<usize> {
        self.search(x).err()
    }
    fn index_to_remove(&self, x: u16) -> Option<usize> {
        self.search(x).ok()
    }

    pub fn contains(&self, x: u16) -> bool {
        self.search(x).is_ok()
    }

    pub fn insert(&mut self, x: u16) -> bool {
        if let Some(pos) = self.index_to_insert(x) {
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
        if let Some(pos) = self.index_to_remove(x) {
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
                    self.ranges[pos] = (i + 1)...j;
                    debug_assert!((i + 1) <= j);
                }
                (i, j) if j == x => {
                    self.ranges[pos] = i...(j - 1);
                    debug_assert!(i <= (j - 1));
                }
                _ => unreachable!(),
            };
            true
        } else {
            false
        }
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

macro_rules! impl_PairwiseWith {
    ( $( ( $op:ident, $fn_with:ident, $fn:ident ) ),* ) => ($(
        impl<'a> $op<&'a Rle16> for Rle16 {
            fn $fn_with(&mut self, rle16: &'a Rle16) {
                *self = (&*self).$fn(rle16);
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

impl_PairwiseWith!(
    (IntersectionWith, intersection_with, intersection),
    (UnionWith, union_with, union),
    (DifferenceWith, difference_with, difference),
    (
        SymmetricDifferenceWith,
        symmetric_difference_with,
        symmetric_difference
    )
);

impl ::Rank<u16> for Rle16 {
    type Weight = u32;

    const SIZE: Self::Weight = super::CAPACITY as u32;

    fn rank1(&self, i: u16) -> Self::Weight {
        if i as usize >= super::CAPACITY {
            return self.count_ones();
        }
        match self.search(i) {
            Err(n) => {
                if n >= self.ranges.len() {
                    self.weight
                } else {
                    self.ranges
                        .iter()
                        .map(|r| (r.end - r.start) as u32 + 1)
                        .take(n)
                        .sum::<u32>()
                }
            }
            Ok(n) => {
                let r = self.ranges
                    .iter()
                    .map(|r| (r.end - r.start) as u32 + 1)
                    .take(n)
                    .sum::<u32>();
                (i as u32 - (self.ranges[n].start as u32)) + r + 1
            }
        }
    }

    fn rank0(&self, i: u16) -> Self::Weight {
        i as Self::Weight + 1 - self.rank1(i)
    }
}

impl ::Select1<u16> for Rle16 {
    fn select1(&self, c: u16) -> Option<u16> {
        if c as u32 >= self.count_ones() {
            return None;
        }
        let mut curr = 0;
        for range in &self.ranges {
            let next = curr + (range.end - range.start + 1);
            if next > c {
                return Some(range.start - curr + c);
            }
            curr = next;
        }
        None
    }
}

impl ::Select0<u16> for Rle16 {
    fn select0(&self, c: u16) -> Option<u16> {
        let c32 = c as u32;
        if c as u32 >= self.count_zeros() {
            return None;
        }

        let pos = self.ranges
            .binary_search_by(|ri| self.rank0(ri.start).cmp(&c32));

        let rank1 = match pos {
            Err(i) if i == 0 => 0,
            Err(i) => self.rank1(self.ranges[i - 1].end),
            Ok(i) => self.rank1(self.ranges[i].end),
        } as u16;
        Some(c + rank1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

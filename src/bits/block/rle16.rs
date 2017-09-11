use std::ops::{Range, RangeInclusive};
use std::{cmp, mem, u16};
use itertools;

use bits::pair::*;
use super::{Block, Rle16, Seq16, Seq64};

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
        self.ranges
            .binary_search_by(|range| if range.start <= *x && *x <= range.end {
                Ordering::Equal
            } else if *x < range.start {
                Ordering::Greater
            } else if range.end < *x {
                Ordering::Less
            } else {
                unreachable!()
            })
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
        const WIDTH: u16 = 64;
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
            let w = u32::from(r.end - r.start) + 1;
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
                let (weight, ranges) = repair(fold);
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum BelongTo<T> {
    None(Range<T>),
    Lhs(Range<T>),
    Rhs(Range<T>),
    Both(Range<T>),
}
impl<T: Copy> BelongTo<T> {
    pub fn range(&self) -> Range<T> {
        match *self {
            BelongTo::None(ref r) |
            BelongTo::Lhs(ref r) |
            BelongTo::Rhs(ref r) |
            BelongTo::Both(ref r) => r.start..r.end,
        }
    }
}

pub(crate) type Ranges<T = u16> = [RangeInclusive<T>];

pub(crate) struct TwoFold<'r, T> {
    lhs: Option<State<T>>,
    rhs: Option<State<T>>,
    window: Box<Iterator<Item = (Boundary<T>, Boundary<T>)> + 'r>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum State<T> {
    Open(T),
    Close(T),
}
use self::State::*;

impl<T> State<T> {
    fn is_open(&self) -> bool {
        match *self {
            Open(_) => true,
            Close(_) => false,
        }
    }
}

// half open
#[derive(Debug, Clone, PartialEq, Eq)]
enum Boundary<T> {
    Lhs(State<T>),
    Rhs(State<T>),
}
use self::Boundary::*;

impl<T: Copy> Boundary<T> {
    fn value(&self) -> T {
        match *self {
            Lhs(Open(i)) | Lhs(Close(i)) | Rhs(Open(i)) | Rhs(Close(i)) => i,
        }
    }
}
impl<'a, T: Ord + Copy> PartialOrd<Boundary<T>> for Boundary<T> {
    fn partial_cmp(&self, rhs: &Boundary<T>) -> Option<cmp::Ordering> {
        Some(self.value().cmp(&rhs.value()))
    }
}
impl<'a, T: Ord + Copy> Ord for Boundary<T> {
    fn cmp(&self, rhs: &Boundary<T>) -> cmp::Ordering {
        self.value().cmp(&rhs.value())
    }
}

impl<'r, T> TwoFold<'r, T> {
    fn lhs_swap(&mut self, lhs: State<T>) {
        self.lhs = Some(lhs);
    }
    fn rhs_swap(&mut self, rhs: State<T>) {
        self.rhs = Some(rhs);
    }

    fn lhs_is_open(&self) -> bool {
        self.lhs.as_ref().map_or(false, |s| s.is_open())
    }
    fn rhs_is_open(&self) -> bool {
        self.rhs.as_ref().map_or(false, |s| s.is_open())
    }
}

// assume that each elements (range) has no overlap
fn merge<'a, 'b, 'r>(lhs: &'a Ranges, rhs: &'b Ranges) -> impl Iterator<Item = Boundary<u32>> + 'r
where
    'a: 'r,
    'b: 'r,
{
    let lhs_iter = lhs.iter().map(to_exclusive).flat_map(|range| {
        vec![
            Boundary::Lhs(Open(range.start)),
            Boundary::Lhs(Close(range.end)),
        ]
    });

    let rhs_iter = rhs.iter().map(to_exclusive).flat_map(|range| {
        vec![
            Boundary::Rhs(Open(range.start)),
            Boundary::Rhs(Close(range.end)),
        ]
    });

    itertools::merge(lhs_iter, rhs_iter)
}

fn to_exclusive(range: &RangeInclusive<u16>) -> Range<u32> {
    let start = u32::from(range.start);
    let end = u32::from(range.end);
    (start..(end + 1))
}

impl<'r> TwoFold<'r, u32> {
    // assume that each elements (range) has no overlap
    pub fn new<'a, 'b>(l: &'a Ranges, r: &'b Ranges) -> TwoFold<'r, u32>
    where
        'a: 'r,
        'b: 'r,
    {
        use itertools::Itertools;

        let window = {
            let merged = merge(l, r);
            let window = merged.tuple_windows();
            Box::new(window)
        };

        let lhs = None;
        let rhs = None;

        TwoFold { lhs, rhs, window }
    }

    pub fn intersection(self) -> impl Iterator<Item = Range<u32>> + 'r {
        self.filter_map(|be| match be {
            BelongTo::Both(_) => Some(be.range()),
            _ => None,
        })
    }

    pub fn union(self) -> impl Iterator<Item = Range<u32>> + 'r {
        self.filter_map(|be| match be {
            BelongTo::None(_) => None,
            _ => Some(be.range()),
        })
    }

    pub fn difference(self) -> impl Iterator<Item = Range<u32>> + 'r {
        self.filter_map(|be| match be {
            BelongTo::Lhs(_) => Some(be.range()),
            _ => None,
        })
    }

    pub fn symmetric_difference(self) -> impl Iterator<Item = Range<u32>> + 'r {
        self.filter_map(|be| match be {
            BelongTo::Lhs(_) | BelongTo::Rhs(_) => Some(be.range()),
            _ => None,
        })
    }
}

/// Repair broken ranges, and accumulate weight.
pub fn repair<I>(folded: I) -> (u32, Vec<RangeInclusive<u16>>)
where
    I: IntoIterator<Item = Range<u32>>,
{
    let mut vec = Vec::new();
    let mut w = 0;
    for curr in folded {
        // doesn't allow value like `3..2`
        assert!(curr.start < curr.end);

        w += curr.end - curr.start;

        let start = curr.start as u16;
        let end = (curr.end - 1) as u16;

        if vec.is_empty() {
            vec.push(start...end);
            continue;
        }

        let i = vec.len();

        // leap should not happen
        assert!(vec[i - 1].end <= start);

        if start == (vec[i - 1].end + 1) {
            // merge into a previous range
            vec[i - 1] = vec[i - 1].start...end;
        } else {
            vec.push(start...end);
        }
    }
    (w, vec)
}

macro_rules! filter {
    ( $i:expr, $j:expr, $r:expr ) => {
        if $i == $j {
            continue;
        } else {
            return Some($r($i..$j));
        }
    }
}

impl<'r> Iterator for TwoFold<'r, u32> {
    type Item = BelongTo<u32>;
    fn next(&mut self) -> Option<BelongTo<u32>> {
        while let Some(next) = self.window.next() {
            match next {
                (Lhs(Open(i)), Rhs(Open(j))) => {
                    self.lhs_swap(Open(i));
                    self.rhs_swap(Open(j));
                    filter!(i, j, BelongTo::Lhs)
                }

                (Lhs(Open(i)), Lhs(Close(j))) => {
                    let belong_to = if self.rhs_is_open() {
                        BelongTo::Both(i..j)
                    } else {
                        BelongTo::Lhs(i..j)
                    };
                    self.lhs_swap(Close(j));
                    return Some(belong_to);
                }

                (Lhs(Open(i)), Rhs(Close(j))) => {
                    self.lhs_swap(Open(i));
                    self.rhs_swap(Close(j));
                    filter!(i, j, BelongTo::Both)
                }

                (Lhs(Close(i)), Lhs(Open(j))) => {
                    let belong_to = if self.rhs_is_open() {
                        BelongTo::Rhs(i..j)
                    } else {
                        BelongTo::None(i..j)
                    };
                    self.lhs_swap(Open(j));
                    return Some(belong_to);
                }

                (Lhs(Close(i)), Rhs(Open(j))) => {
                    self.lhs_swap(Close(i));
                    self.rhs_swap(Open(j));
                    filter!(i, j, BelongTo::None)
                }

                (Lhs(Close(i)), Rhs(Close(j))) => {
                    self.lhs_swap(Close(i));
                    self.rhs_swap(Close(j));
                    filter!(i, j, BelongTo::Rhs)
                }

                (Rhs(Open(i)), Lhs(Open(j))) => {
                    self.lhs_swap(Open(j));
                    self.rhs_swap(Open(i));
                    filter!(i, j, BelongTo::Rhs)
                }

                (Rhs(Open(i)), Lhs(Close(j))) => {
                    self.lhs_swap(Close(j));
                    self.rhs_swap(Open(i));
                    filter!(i, j, BelongTo::Both)
                }

                (Rhs(Open(i)), Rhs(Close(j))) => {
                    let belong_to = if self.lhs_is_open() {
                        BelongTo::Both(i..j)
                    } else {
                        BelongTo::Rhs(i..j)
                    };
                    self.rhs_swap(Close(j));
                    return Some(belong_to);
                }

                (Rhs(Close(i)), Lhs(Open(j))) => {
                    self.lhs_swap(Open(j));
                    self.rhs_swap(Close(i));
                    filter!(i, j, BelongTo::None)
                }

                (Rhs(Close(i)), Lhs(Close(j))) => {
                    self.lhs_swap(Close(j));
                    self.rhs_swap(Close(i));
                    filter!(i, j, BelongTo::Lhs)
                }

                (Rhs(Close(i)), Rhs(Open(j))) => {
                    let belong_to = if self.lhs_is_open() {
                        BelongTo::Lhs(i..j)
                    } else {
                        BelongTo::None(i..j)
                    };
                    self.rhs_swap(Open(j));
                    return Some(belong_to);
                }

                (Lhs(Open(_)), Lhs(Open(_))) => unreachable!(),
                (Rhs(Open(_)), Rhs(Open(_))) => unreachable!(),
                (Lhs(Close(_)), Lhs(Close(_))) => unreachable!(),
                (Rhs(Close(_)), Rhs(Close(_))) => unreachable!(),
            }
        }
        None
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
    fn insert_remove() {
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

    #[test]
    fn two_fold() {
        static LHS: &Ranges = &[3...5, 10...13, 18...19, 100...120];
        static RHS: &Ranges = &[2...3, 6...9, 12...14, 17...21, 200...1000];
        static NULL: &Ranges = &[];
        static FULL: &Ranges = &[0...u16::MAX];

        assert_eq!(
            TwoFold::new(LHS, RHS).collect::<Vec<BelongTo<u32>>>(),
            vec![
                BelongTo::Rhs(2..3),
                BelongTo::Both(3..4),
                BelongTo::Lhs(4..6),
                BelongTo::Rhs(6..10),
                BelongTo::Lhs(10..12),
                BelongTo::Both(12..14),
                BelongTo::Rhs(14..15),
                BelongTo::None(15..17),
                BelongTo::Rhs(17..18),
                BelongTo::Both(18..20),
                BelongTo::Rhs(20..22),
                BelongTo::None(22..100),
                BelongTo::Lhs(100..121),
                BelongTo::None(121..200),
                BelongTo::Rhs(200..1001),
            ]
        );

        assert_eq!(
            TwoFold::new(NULL, RHS).collect::<Vec<BelongTo<u32>>>(),
            vec![
                BelongTo::Rhs(2..4),
                BelongTo::None(4..6),
                BelongTo::Rhs(6..10),
                BelongTo::None(10..12),
                BelongTo::Rhs(12..15),
                BelongTo::None(15..17),
                BelongTo::Rhs(17..22),
                BelongTo::None(22..200),
                BelongTo::Rhs(200..1001),
            ]
        );

        assert_eq!(
            TwoFold::new(LHS, NULL).collect::<Vec<BelongTo<u32>>>(),
            vec![
                BelongTo::Lhs(3..6),
                BelongTo::None(6..10),
                BelongTo::Lhs(10..14),
                BelongTo::None(14..18),
                BelongTo::Lhs(18..20),
                BelongTo::None(20..100),
                BelongTo::Lhs(100..121),
            ]
        );

        assert_eq!(
            TwoFold::new(FULL, RHS).collect::<Vec<BelongTo<u32>>>(),
            vec![
                BelongTo::Lhs(0..2),
                BelongTo::Both(2..4),
                BelongTo::Lhs(4..6),
                BelongTo::Both(6..10),
                BelongTo::Lhs(10..12),
                BelongTo::Both(12..15),
                BelongTo::Lhs(15..17),
                BelongTo::Both(17..22),
                BelongTo::Lhs(22..200),
                BelongTo::Both(200..1001),
                BelongTo::Lhs(1001..65536),
            ]
        );

        assert_eq!(
            TwoFold::new(LHS, FULL).collect::<Vec<BelongTo<u32>>>(),
            vec![
                BelongTo::Rhs(0..3),
                BelongTo::Both(3..6),
                BelongTo::Rhs(6..10),
                BelongTo::Both(10..14),
                BelongTo::Rhs(14..18),
                BelongTo::Both(18..20),
                BelongTo::Rhs(20..100),
                BelongTo::Both(100..121),
                BelongTo::Rhs(121..65536),
            ]
        );

        let a1 = &[0...1, 3...5, 12...16, 18...19];
        let a2 = &[0...0, 3...8, 10...13, 15...15, 19...19];

        assert_eq!(
            TwoFold::new(a1, a2).collect::<Vec<BelongTo<u32>>>(),
            vec![
                BelongTo::Both(0..1),
                BelongTo::Lhs(1..2),
                BelongTo::None(2..3),
                BelongTo::Both(3..6),
                BelongTo::Rhs(6..9),
                BelongTo::None(9..10),
                BelongTo::Rhs(10..12),
                BelongTo::Both(12..14),
                BelongTo::Lhs(14..15),
                BelongTo::Both(15..16),
                BelongTo::Lhs(16..17),
                BelongTo::None(17..18),
                BelongTo::Lhs(18..19),
                BelongTo::Both(19..20),
            ]
        );

        assert_eq!(
            TwoFold::new(a2, a1).collect::<Vec<BelongTo<u32>>>(),
            vec![
                BelongTo::Both(0..1),
                BelongTo::Rhs(1..2),
                BelongTo::None(2..3),
                BelongTo::Both(3..6),
                BelongTo::Lhs(6..9),
                BelongTo::None(9..10),
                BelongTo::Lhs(10..12),
                BelongTo::Both(12..14),
                BelongTo::Rhs(14..15),
                BelongTo::Both(15..16),
                BelongTo::Rhs(16..17),
                BelongTo::None(17..18),
                BelongTo::Rhs(18..19),
                BelongTo::Both(19..20),
            ]
        );
    }
}

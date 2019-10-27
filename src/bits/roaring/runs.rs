use std::{
    iter::{FromIterator, Peekable},
    ops::{Range, RangeInclusive},
    slice,
};

use crate::{
    bits::{Difference, Intersection, SymmetricDifference, Union},
    num::try_cast,
    ops::*,
};

use super::{Block, Ordering, Run, Runs, EQ, GT, LT};

impl<'a> IntoIterator for &'a Runs {
    type Item = &'a Run;
    type IntoIter = slice::Iter<'a, Run>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.iter()
    }
}

impl IntoIterator for Runs {
    type Item = Run;
    type IntoIter = std::vec::IntoIter<Run>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl IntoIterator for Run {
    type Item = u16;
    type IntoIter = RangeInclusive<u16>;
    fn into_iter(self) -> Self::IntoIter {
        RangeInclusive::new(self.0, self.1)
    }
}

impl Run {
    fn start(&self) -> &u16 {
        &self.0
    }
    fn end(&self) -> &u16 {
        &self.1
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    #[inline]
    fn len(&self) -> usize {
        try_cast::<u16, usize>(self.1 - self.0) + 1
    }
}

impl Runs {
    #[inline]
    fn search_bounds(&self, n: u16) -> Result<usize, usize> {
        self.data.binary_search_by(|&Run(i, j)| {
            if i <= n && n <= j {
                EQ
            } else if n < i {
                GT
            } else {
                LT
            }
        })
    }

    #[inline]
    fn index_to_insert(&self, i: u16) -> Option<usize> {
        self.search_bounds(i).err()
    }

    #[inline]
    fn index_to_remove(&self, i: u16) -> Option<usize> {
        self.search_bounds(i).ok()
    }
}

// impl<u16: Word> BitBlock for Runs<u16> {
//     const BITS: u64 = u16::CAu16;
//     fn empty() -> Self {
//         Self::default()
//     }
// }

impl Bits for Runs {
    #[inline]
    fn size(&self) -> usize {
        Block::BITS
    }

    #[inline]
    fn count1(&self) -> usize {
        self.data.iter().map(Run::len).sum()
    }

    #[inline]
    fn any(&self) -> bool {
        !self.data.is_empty()
    }

    #[inline]
    fn bit(&self, i: usize) -> bool {
        self.search_bounds(try_cast(i)).is_ok()
    }
}

impl BitsMut for Runs {
    fn put1(&mut self, i: usize) -> &mut Self {
        let i = try_cast(i);
        if let Some(pos) = self.index_to_insert(i) {
            let runs = &mut self.data;
            let run_lhs = if pos > 0 {
                Some(*runs[pos - 1].end())
            } else {
                None
            }; // should be get_mut?
            let run_rhs = runs.get(pos).map(|r| *r.start());

            match (run_lhs, run_rhs) {
                (Some(lhs), Some(rhs)) if lhs + 1 == i && i == rhs - 1 => {
                    let start = *runs[pos - 1].start();
                    let end = *runs[pos].end();
                    runs[pos - 1] = Run(start, end);
                    runs.remove(pos);
                }

                (Some(lhs), None) if lhs + 1 == i => {
                    let start = *runs[pos - 1].start();
                    let end = *runs[pos - 1].end() + 1;
                    runs[pos - 1] = Run(start, end);
                }

                (None, Some(rhs)) if i == rhs - 1 => {
                    let start = *runs[pos].start() - 1;
                    let end = *runs[pos].end();
                    runs[pos] = Run(start, end);
                }

                _ => {
                    runs.insert(pos, Run(i, i));
                }
            }
        }
        self
    }

    fn put0(&mut self, i: usize) -> &mut Self {
        let i = try_cast(i);
        if let Some(pos) = self.index_to_remove(i) {
            let runs = &mut self.data;

            match runs[pos] {
                Run(n, m) if n == m => {
                    runs.remove(pos);
                }

                Run(n, m) if n < i && i < m => {
                    runs[pos] = Run(n, i - 1);
                    runs.insert(pos + 1, Run(i + 1, m));
                }

                Run(n, m) if i == n => {
                    assert!(n < m);
                    runs[pos] = Run(n + 1, m);
                }

                Run(n, m) if i == m => {
                    assert!(n < m);
                    runs[pos] = Run(n, m - 1);
                }

                _ => unreachable!(),
            };
        }
        self
    }
}

impl BitRank for Runs {
    fn rank1(&self, i: usize, j: usize) -> usize {
        let rank = |i| {
            let iter = self.data.iter().map(Run::len);
            match self.search_bounds(try_cast(i)) {
                Ok(n) => {
                    iter.take(n).sum::<usize>() + i - try_cast::<u16, usize>(*self.data[n].start())
                }
                Err(n) => iter.take(n).sum(),
            }
        };

        let bits = self.size();
        match (i, j) {
            (i, j) if i == j => 0,
            (0, k) if k == bits => self.count1(),
            (0, k) => rank(k),
            (i, j) => rank(j) - rank(i),
        }
    }
}

impl BitSelect for Runs {
    fn select1(&self, c: usize) -> Option<usize> {
        let mut curr = 0;
        for run in &self.data {
            let next = curr + run.len();
            if next > c {
                return Some(try_cast::<u16, usize>(*run.start()) - curr + c);
            }
            curr = next;
        }
        None
    }
}

impl FromIterator<Range<usize>> for Runs {
    fn from_iter<I>(iterable: I) -> Self
    where
        I: IntoIterator<Item = Range<usize>>,
    {
        let mut data = Vec::new();
        for range in iterable {
            let s = try_cast(range.start);
            let e = try_cast(range.end - 1);
            assert!(s <= e);

            // 1st time
            if data.is_empty() {
                data.push(Run(s, e));
            } else {
                let last = data.len() - 1;
                let Run(x, y) = data[last];
                assert!(y <= s); // no overlap

                // panics if y is a max value of `u16`.
                // this doesn't happen on a valid iterator.
                assert_ne!(y, std::u16::MAX);

                if s == y + 1 {
                    // merge into a previous range
                    data[last] = Run(x, e);
                } else {
                    data.push(Run(s, e));
                }
            }
        }
        Runs { data }
    }
}

#[derive(Debug, Clone, Copy)]
enum Braket {
    Bra(usize), // [
    Ket(usize), // )
}
use Braket::*;

#[derive(Debug, Clone, Copy)]
enum Side {
    Lhs(Braket),
    Rhs(Braket),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Region {
    Lhs(Range<usize>), // L and not R
    Rhs(Range<usize>), // R and not L
    And(Range<usize>), // L and R
    Not(Range<usize>), // not (L or R)
}

impl PartialEq<Braket> for Braket {
    fn eq(&self, rhs: &Braket) -> bool {
        self.value().eq(&rhs.value())
    }
}
impl Eq for Braket {}

impl PartialOrd<Braket> for Braket {
    fn partial_cmp(&self, rhs: &Braket) -> Option<Ordering> {
        self.value().partial_cmp(&rhs.value())
    }
}
impl Ord for Braket {
    fn cmp(&self, rhs: &Braket) -> Ordering {
        self.value().cmp(&rhs.value())
    }
}

impl Braket {
    #[inline]
    fn is_bra(&self) -> bool {
        match self {
            Bra(_) => true,
            _ => false,
        }
    }

    // #[inline]
    // fn is_ket(&self) -> bool {
    //     match self {
    //         Ket(_) => true,
    //         _ => false,
    //     }
    // }

    #[inline]
    fn value(&self) -> usize {
        use Braket::*;
        match self {
            Bra(v) | Ket(v) => *v,
        }
    }
}

impl Region {
    fn bounds(&self) -> &Range<usize> {
        match self {
            Region::Lhs(r) | Region::Rhs(r) | Region::And(r) | Region::Not(r) => r,
        }
    }
    fn is_empty(&self) -> bool {
        let r = self.bounds();
        r.end - r.start == 0
    }
}

struct Regions<I: Iterator<Item = Region>> {
    finished: bool,
    max_size: usize,
    last_val: Option<Region>,
    regions: Peekable<I>,
}

impl<I: Iterator<Item = Region>> Regions<I> {
    fn into_and(self) -> impl Iterator<Item = Range<usize>> {
        self.filter_map(|member| match member {
            Region::And(range) => Some(range),
            _ => None,
        })
    }

    fn into_or(self) -> impl Iterator<Item = Range<usize>> {
        self.filter_map(|member| match member {
            Region::Lhs(range) => Some(range),
            Region::Rhs(range) => Some(range),
            Region::And(range) => Some(range),
            _ => None,
        })
    }

    fn into_and_not(self) -> impl Iterator<Item = Range<usize>> {
        self.filter_map(|member| match member {
            Region::Lhs(range) => Some(range),
            _ => None,
        })
    }

    fn into_xor(self) -> impl Iterator<Item = Range<usize>> {
        self.filter_map(|member| match member {
            Region::Lhs(range) | Region::Rhs(range) => Some(range),
            _ => None,
        })
    }

    // pub fn into_not(self) -> impl Iterator<Item = Range<usize>> + 'r {
    //     self.filter_map(|member| match member {
    //         Region::Not(range) => Some(range),
    //         _ => None,
    //     })
    // }
}

impl<I: Iterator<Item = Region>> Iterator for Regions<I> {
    type Item = Region;
    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        };

        loop {
            let peek = self.regions.peek();
            match (self.last_val.clone(), peek) {
                // `inner_region` may yields empty value
                (_, Some(region)) if region.is_empty() => {
                    self.regions.next().unwrap();
                    continue;
                }

                (None, Some(head)) => {
                    let range = head.bounds();
                    let s = range.start;
                    if s == 0 {
                        self.last_val = self.regions.next();
                        return self.last_val.clone();
                    } else {
                        self.last_val = Some(Region::Not(0..s));
                        return self.last_val.clone();
                    }
                }

                (Some(_), Some(next)) => {
                    let out = Some(next.clone());
                    self.last_val = self.regions.next();
                    return out;
                }

                (Some(last), None) => {
                    self.finished = true;
                    let range = last.bounds();
                    let e = range.end;
                    if e < self.max_size {
                        return Some(Region::Not(e..self.max_size));
                    } else {
                        return None;
                    }
                }

                // iterator is empty
                (None, None) => {
                    self.finished = true;
                    return Some(Region::Not(0..self.max_size));
                }
            }
        }
    }
}

fn regions<'a: 'r, 'b: 'r, 'r>(
    this: impl IntoIterator<Item = &'a Run> + 'a,
    that: impl IntoIterator<Item = &'b Run> + 'b,
) -> Regions<impl Iterator<Item = Region> + 'r> {
    let max_size = Block::BITS;
    let finished = false;
    let last_val = None;
    let regions = inner_regions(this, that).peekable();
    Regions {
        finished,
        max_size,
        last_val,
        regions,
    }
}

fn merge<'a: 'r, 'b: 'r, 'r>(
    this: impl IntoIterator<Item = &'a Run> + 'a,
    that: impl IntoIterator<Item = &'b Run> + 'b,
) -> impl Iterator<Item = Side> + 'r {
    use {Braket::*, Side::*};

    struct MergeBy<L, R, F>
    where
        L: Iterator,
        R: Iterator,
        F: Fn(&L::Item, &R::Item) -> Ordering,
    {
        lhs: Peekable<L>,
        rhs: Peekable<R>,
        fun: F,
    }

    impl<L, R, F> MergeBy<L, R, F>
    where
        L: Iterator,
        R: Iterator,
        F: Fn(&L::Item, &R::Item) -> Ordering,
    {
        fn merge_by<A, B, T>(lhs: A, rhs: B, fun: F) -> Self
        where
            A: IntoIterator<Item = T, IntoIter = L>,
            B: IntoIterator<Item = T, IntoIter = R>,
            L: Iterator<Item = T>,
            R: Iterator<Item = T>,
            F: Fn(&T, &T) -> Ordering,
        {
            let lhs = lhs.into_iter().peekable();
            let rhs = rhs.into_iter().peekable();
            MergeBy { lhs, rhs, fun }
        }
    }

    impl<L, R, F, T> Iterator for MergeBy<L, R, F>
    where
        L: Iterator<Item = T>,
        R: Iterator<Item = T>,
        F: Fn(&T, &T) -> Ordering,
    {
        type Item = T;
        fn next(&mut self) -> Option<Self::Item> {
            match (self.lhs.peek(), self.rhs.peek()) {
                (Some(lhs), Some(rhs)) => match (self.fun)(lhs, rhs) {
                    Ordering::Less | Ordering::Equal => self.lhs.next(),
                    Ordering::Greater => self.rhs.next(),
                },
                (_, None) => self.lhs.next(),
                (None, _) => self.rhs.next(),
            }
        }
    }

    let lhs = {
        let mut vec_lhs = Vec::new();
        for lhs in this {
            let (n, m) = {
                let Run(n, m) = *lhs;
                (try_cast(n), try_cast::<u16, usize>(m) + 1)
            };
            vec_lhs.push(Lhs(Bra(n)));
            vec_lhs.push(Lhs(Ket(m)));
        }
        vec_lhs
    };

    let rhs = {
        let mut vec_rhs = Vec::new();
        for rhs in that {
            let (n, m) = {
                let Run(n, m) = *rhs;
                (try_cast(n), try_cast::<u16, usize>(m) + 1)
            };
            vec_rhs.push(Rhs(Bra(n)));
            vec_rhs.push(Rhs(Ket(m)));
        }
        vec_rhs
    };

    MergeBy::merge_by(lhs, rhs, |a, b| match (a, b) {
        (Lhs(a), Lhs(b)) => a.cmp(b),
        (Lhs(a), Rhs(b)) => a.cmp(b),
        (Rhs(a), Lhs(b)) => a.cmp(b),
        (Rhs(a), Rhs(b)) => a.cmp(b),
    })
}

fn inner_regions<'a: 'r, 'b: 'r, 'r>(
    this: impl IntoIterator<Item = &'a Run> + 'a,
    that: impl IntoIterator<Item = &'b Run> + 'b,
) -> impl Iterator<Item = Region> + 'r {
    // Tuples yields window
    struct Tuples<I: Iterator> {
        iter: I,
        last: Option<I::Item>,
    }
    impl<I: Iterator> Tuples<I> {
        fn tuples(mut iter: I) -> Self {
            let last = iter.next();
            Tuples { iter, last }
        }
    }
    impl<I> Iterator for Tuples<I>
    where
        I: Iterator,
        I::Item: Copy,
    {
        type Item = (I::Item, I::Item);
        fn next(&mut self) -> Option<Self::Item> {
            if let Some(last) = self.last {
                self.last = self.iter.next();
                if let Some(next) = self.last {
                    return Some((last, next));
                }
            }
            None
        }
    }

    use Side::*;

    let merged = merge(this, that);
    let tuples = Tuples::tuples(merged);

    tuples.scan((Ket(0), Ket(0)), |(lhs, rhs), value| match value {
        (Lhs(Bra(i)), Lhs(Ket(j))) => {
            *lhs = Ket(j);
            Some(if rhs.is_bra() {
                Region::And(i..j)
            } else {
                Region::Lhs(i..j)
            })
        }

        (Lhs(Ket(i)), Lhs(Bra(j))) => {
            *lhs = Bra(j);
            Some(if rhs.is_bra() {
                Region::Rhs(i..j)
            } else {
                Region::Not(i..j)
            })
        }

        (Rhs(Bra(i)), Rhs(Ket(j))) => {
            *rhs = Ket(j);
            Some(if lhs.is_bra() {
                Region::And(i..j)
            } else {
                Region::Rhs(i..j)
            })
        }
        (Rhs(Ket(i)), Rhs(Bra(j))) => {
            *rhs = Bra(j);
            Some(if lhs.is_bra() {
                Region::Lhs(i..j)
            } else {
                Region::Not(i..j)
            })
        }

        (Lhs(Bra(i)), Rhs(Bra(j))) => {
            *lhs = Bra(i);
            *rhs = Bra(j);
            Some(Region::Lhs(i..j))
        }

        (Lhs(Bra(i)), Rhs(Ket(j))) => {
            *lhs = Bra(i);
            *rhs = Ket(j);
            Some(Region::And(i..j))
        }

        (Lhs(Ket(i)), Rhs(Bra(j))) => {
            *lhs = Ket(i);
            *rhs = Bra(j);
            Some(Region::Not(i..j))
        }

        (Lhs(Ket(i)), Rhs(Ket(j))) => {
            *lhs = Ket(i);
            *rhs = Ket(j);
            Some(Region::Rhs(i..j))
        }

        (Rhs(Bra(i)), Lhs(Bra(j))) => {
            *lhs = Bra(j);
            *rhs = Bra(i);
            Some(Region::Rhs(i..j))
        }

        (Rhs(Bra(i)), Lhs(Ket(j))) => {
            *lhs = Ket(j);
            *rhs = Bra(i);
            Some(Region::And(i..j))
        }

        (Rhs(Ket(i)), Lhs(Bra(j))) => {
            *lhs = Bra(j);
            *rhs = Ket(i);
            Some(Region::Not(i..j))
        }

        (Rhs(Ket(i)), Lhs(Ket(j))) => {
            *lhs = Ket(j);
            *rhs = Ket(i);
            Some(Region::Lhs(i..j))
        }

        _ => unreachable!(),
    })
}

impl Intersection<Self> for Runs {
    fn intersection(&mut self, runs: &Self) {
        *self = regions(&self.data, &runs.data).into_and().collect::<Runs>();
    }
}

impl Union<Self> for Runs {
    fn union(&mut self, runs: &Self) {
        *self = regions(&self.data, &runs.data).into_or().collect::<Runs>();
    }
}

impl Difference<Self> for Runs {
    fn difference(&mut self, runs: &Self) {
        *self = regions(&self.data, &runs.data)
            .into_and_not()
            .collect::<Runs>();
    }
}

impl SymmetricDifference<Self> for Runs {
    fn symmetric_difference(&mut self, runs: &Self) {
        *self = regions(&self.data, &runs.data).into_xor().collect::<Runs>();
    }
}

#[cfg(test)]
#[test]
#[rustfmt::skip]
fn test_regions() {
    assert_eq!(
        regions(
            &[Run(3u16, 5), Run(7, 10)],
            &[Run(2, 3)],
        )
        .collect::<Vec<Region>>(),
        vec![
            Region::Not(0..2),
            Region::Rhs(2..3),
            Region::And(3..4),
            Region::Lhs(4..6),
            Region::Not(6..7),
            Region::Lhs(7..11),
            Region::Not(11..65536),
        ]
    );

    assert_eq!(
        regions(
            &[Run(3u16, 5), Run(6, 10)],
            &[Run(2, 3), Run(6, 6)],
        )
        .collect::<Vec<Region>>(),
        vec![
            Region::Not(0..2),
            Region::Rhs(2..3),
            Region::And(3..4),
            Region::Lhs(4..6),
            Region::And(6..7),
            Region::Lhs(7..11),
            Region::Not(11..65536),
        ]
    );

    assert_eq!(
        regions(
            &[Run(3u16, 5), Run(10, 13), Run(18, 19), Run(100, 120)],
            &[],
        )
        .collect::<Vec<Region>>(),
        vec![
            Region::Not(0..3),
            Region::Lhs(3..6),
            Region::Not(6..10),
            Region::Lhs(10..14),
            Region::Not(14..18),
            Region::Lhs(18..20),
            Region::Not(20..100),
            Region::Lhs(100..121),
            Region::Not(121..65536)
        ]
    );

    assert_eq!(
        regions(
            &[Run(3_u16, 5), Run(10, 13), Run(18, 19), Run(100, 120)],
            &[Run(2_u16, 3), Run(6, 9), Run(12, 14), Run(17, 21), Run(200, 999)],
        )
        .collect::<Vec<Region>>(),
        vec![
            Region::Not(0..2),
            Region::Rhs(2..3),
            Region::And(3..4),
            Region::Lhs(4..6),
            Region::Rhs(6..10),
            Region::Lhs(10..12),
            Region::And(12..14),
            Region::Rhs(14..15),
            Region::Not(15..17),
            Region::Rhs(17..18),
            Region::And(18..20),
            Region::Rhs(20..22),
            Region::Not(22..100),
            Region::Lhs(100..121),
            Region::Not(121..200),
            Region::Rhs(200..1000),
            Region::Not(1000..65536),
        ]
    );
}

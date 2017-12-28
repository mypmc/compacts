#![cfg_attr(feature = "cargo-clippy", allow(range_minus_one))]
#![cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]

use std::{cmp, ops, u16};
use bits::pair;
use super::{Range, RunBlock};

impl RunBlock {
    pub fn new() -> Self {
        RunBlock::default()
    }

    pub fn weight(&self) -> u32 {
        self.weight
    }
    pub fn ranges(&self) -> &[Range] {
        &self.ranges
    }

    pub fn search(&self, x: &u16) -> Result<usize, usize> {
        let n = *x;
        self.ranges.binary_search_by(|range| {
            if range.start <= n && n <= range.end {
                cmp::Ordering::Equal
            } else if n < range.start {
                cmp::Ordering::Greater
            } else {
                // range.end < n
                cmp::Ordering::Less
            }
        })
    }

    #[inline]
    fn index_to_insert(&self, x: &u16) -> Option<usize> {
        self.search(x).err()
    }

    #[inline]
    fn index_to_remove(&self, x: &u16) -> Option<usize> {
        self.search(x).ok()
    }

    #[inline]
    pub fn contains(&self, x: u16) -> bool {
        self.search(&x).is_ok()
    }

    pub fn insert(&mut self, x: u16) -> bool {
        let mut inserted = false;
        if let Some(pos) = self.index_to_insert(&x) {
            self.weight += 1;
            inserted = true;

            let lhs_bound = if pos != 0 {
                Some(self.ranges[pos - 1].end)
            } else {
                None
            };
            let rhs_bound = if pos < self.ranges.len() {
                Some(self.ranges[pos].start)
            } else {
                None
            };

            match (lhs_bound, rhs_bound) {
                // connect lhs and rhs
                (Some(lhs), Some(rhs)) if lhs + 1 == x && x == rhs - 1 => {
                    let start = self.ranges[pos - 1].start;
                    let end = self.ranges[pos].end;
                    self.ranges[pos - 1] = start..=end;
                    self.ranges.remove(pos);
                }
                // extend lhs
                (Some(lhs), None) if lhs + 1 == x => {
                    let start = self.ranges[pos - 1].start;
                    let end = self.ranges[pos - 1].end + 1;
                    self.ranges[pos - 1] = start..=end;
                }
                // extend rhs
                (None, Some(rhs)) if x == rhs - 1 => {
                    let start = self.ranges[pos].start - 1;
                    let end = self.ranges[pos].end;
                    self.ranges[pos] = start..=end;
                }
                _ => {
                    self.ranges.insert(pos, x..=x);
                }
            }
        }
        !inserted
    }

    pub fn remove(&mut self, x: u16) -> bool {
        let mut removed = false;
        if let Some(pos) = self.index_to_remove(&x) {
            self.weight -= 1;
            removed = true;

            match (self.ranges[pos].start, self.ranges[pos].end) {
                (i, j) if i == j => {
                    self.ranges.remove(pos);
                }
                (i, j) if i < x && x < j => {
                    self.ranges[pos] = i..=(x - 1);
                    self.ranges.insert(pos + 1, (x + 1)..=j);
                }
                (i, j) if i == x => {
                    assert!(i + 1 <= j);
                    self.ranges[pos] = (i + 1)..=j;
                }
                (i, j) if j == x => {
                    assert!(i <= j - 1);
                    self.ranges[pos] = i..=(j - 1);
                }
                _ => unreachable!(),
            };
        }
        removed
    }

    pub fn and<'a, 'b, 'r>(
        &'a self,
        that: &'b RunBlock,
    ) -> impl Iterator<Item = ops::Range<u32>> + 'r
    where
        'a: 'r,
        'b: 'r,
    {
        let it = Overlap::new(&self.ranges, &that.ranges);
        it.filter_map(filter_and)
    }

    pub fn or<'a, 'b, 'r>(
        &'a self,
        that: &'b RunBlock,
    ) -> impl Iterator<Item = ops::Range<u32>> + 'r
    where
        'a: 'r,
        'b: 'r,
    {
        let it = Overlap::new(&self.ranges, &that.ranges);
        it.filter_map(filter_or)
    }

    pub fn and_not<'a, 'b, 'r>(
        &'a self,
        that: &'b RunBlock,
    ) -> impl Iterator<Item = ops::Range<u32>> + 'r
    where
        'a: 'r,
        'b: 'r,
    {
        let it = Overlap::new(&self.ranges, &that.ranges);
        it.filter_map(filter_and_not)
    }

    pub fn xor<'a, 'b, 'r>(
        &'a self,
        that: &'b RunBlock,
    ) -> impl Iterator<Item = ops::Range<u32>> + 'r
    where
        'a: 'r,
        'b: 'r,
    {
        let it = Overlap::new(&self.ranges, &that.ranges);
        it.filter_map(filter_xor)
    }
}

struct Overlap<'r, T> {
    lhs_ck: bool, // track whether last lhs section is open
    rhs_ck: bool, // track whether last rhs section is open
    tuples: Box<Iterator<Item = (Boundary<T>, Boundary<T>)> + 'r>,
}
struct Tuples<I: Iterator> {
    iter: I,
    last: Option<I::Item>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum BelongTo<T> {
    Lhs(ops::Range<T>),
    Rhs(ops::Range<T>),
    Both(ops::Range<T>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Boundary<T> {
    Lhs(Section<T>),
    Rhs(Section<T>),
}
#[derive(Debug, Clone, PartialEq, Eq)]
enum Section<T> {
    Start(T),
    End(T),
}

impl<T: Copy> BelongTo<T> {
    fn inner(&self) -> &ops::Range<T> {
        match *self {
            BelongTo::Both(ref r) | BelongTo::Lhs(ref r) | BelongTo::Rhs(ref r) => r,
        }
    }
    fn range(&self) -> ops::Range<T> {
        self.inner().clone()
    }
}

impl<I: Iterator> Tuples<I> {
    fn new(mut iter: I) -> Self {
        let last = iter.next();
        Tuples { iter, last }
    }
}
impl<I> Iterator for Tuples<I>
where
    I: Iterator,
    I::Item: Clone,
{
    type Item = (I::Item, I::Item);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(last) = self.last.clone() {
            self.last = self.iter.next();
            if let Some(next) = self.last.clone() {
                return Some((last, next));
            }
        }
        None
    }
}

impl<T: Copy> Boundary<T> {
    fn value(&self) -> T {
        use self::Boundary::*;
        use self::Section::*;
        match *self {
            Lhs(Start(i)) | Lhs(End(i)) | Rhs(Start(i)) | Rhs(End(i)) => i,
        }
    }
}
impl<'a, T: Ord + Copy> PartialOrd<Boundary<T>> for Boundary<T> {
    fn partial_cmp(&self, rhs: &Boundary<T>) -> Option<cmp::Ordering> {
        self.value().partial_cmp(&rhs.value())
    }
}
impl<'a, T: Ord + Copy> Ord for Boundary<T> {
    fn cmp(&self, rhs: &Boundary<T>) -> cmp::Ordering {
        self.value().cmp(&rhs.value())
    }
}

// assume that each elements (range) has no overlap
fn merge<'a, 'b, 'r>(lhs: &'a [Range], rhs: &'b [Range]) -> impl Iterator<Item = Boundary<u32>> + 'r
where
    'a: 'r,
    'b: 'r,
{
    let lhs_iter = lhs.iter().flat_map(|range| {
        let range = to_exclusive(range);
        vec![
            Boundary::Lhs(Section::Start(range.start)),
            Boundary::Lhs(Section::End(range.end)),
        ]
    });
    let rhs_iter = rhs.iter().flat_map(|range| {
        let range = to_exclusive(range);
        vec![
            Boundary::Rhs(Section::Start(range.start)),
            Boundary::Rhs(Section::End(range.end)),
        ]
    });
    pair::merge(lhs_iter, rhs_iter)
}

fn to_exclusive(range: &Range) -> ops::Range<u32> {
    let start = u32::from(range.start);
    let end = u32::from(range.end);
    start..(end + 1)
}

impl<'r> Overlap<'r, u32> {
    pub fn new<'a, 'b>(lhs: &'a [Range], rhs: &'b [Range]) -> Overlap<'r, u32>
    where
        'a: 'r,
        'b: 'r,
    {
        let lhs_ck = false;
        let rhs_ck = false;
        let tuples = {
            let merged = merge(lhs, rhs);
            let tuples = Tuples::new(merged);
            Box::new(tuples)
        };
        Overlap {
            lhs_ck,
            rhs_ck,
            tuples,
        }
    }
}

fn filter_and(be: BelongTo<u32>) -> Option<ops::Range<u32>> {
    match be {
        BelongTo::Both(_) => Some(be.range()),
        _ => None,
    }
}

fn filter_or(be: BelongTo<u32>) -> Option<ops::Range<u32>> {
    Some(be.range())
}

fn filter_and_not(be: BelongTo<u32>) -> Option<ops::Range<u32>> {
    match be {
        BelongTo::Lhs(_) => Some(be.range()),
        _ => None,
    }
}

fn filter_xor(be: BelongTo<u32>) -> Option<ops::Range<u32>> {
    match be {
        BelongTo::Lhs(_) | BelongTo::Rhs(_) => Some(be.range()),
        _ => None,
    }
}

macro_rules! belongck {
    ( $i:expr, $j:expr, $belong:expr ) => {
        if $i == $j {
            continue;
        } else {
            return Some($belong($i..$j));
        }
    }
}

impl<'r> Iterator for Overlap<'r, u32> {
    type Item = BelongTo<u32>;
    fn next(&mut self) -> Option<BelongTo<u32>> {
        use self::Boundary::*;
        use self::Section::*;

        while let Some(next) = self.tuples.next() {
            match next {
                (Lhs(Start(i)), Rhs(Start(j))) => {
                    self.lhs_ck = true;
                    self.rhs_ck = true;
                    belongck!(i, j, BelongTo::Lhs)
                }

                (Lhs(Start(i)), Lhs(End(j))) => {
                    let belong_to = if self.rhs_ck {
                        BelongTo::Both(i..j)
                    } else {
                        BelongTo::Lhs(i..j)
                    };
                    self.lhs_ck = false;
                    return Some(belong_to);
                }

                (Lhs(Start(i)), Rhs(End(j))) => {
                    self.lhs_ck = true;
                    self.rhs_ck = false;
                    belongck!(i, j, BelongTo::Both)
                }

                (Lhs(End(i)), Lhs(Start(j))) => {
                    let belong_to = if self.rhs_ck {
                        BelongTo::Rhs(i..j)
                    } else {
                        // BelongTo::None(i..j)
                        continue;
                    };
                    self.lhs_ck = true;
                    return Some(belong_to);
                }

                (Lhs(End(_)), Rhs(Start(_))) => {
                    self.lhs_ck = false;
                    self.rhs_ck = true;
                    continue;
                }

                (Lhs(End(i)), Rhs(End(j))) => {
                    self.lhs_ck = false;
                    self.rhs_ck = false;
                    belongck!(i, j, BelongTo::Rhs)
                }

                (Rhs(Start(i)), Lhs(Start(j))) => {
                    self.lhs_ck = true;
                    self.rhs_ck = true;
                    belongck!(i, j, BelongTo::Rhs)
                }

                (Rhs(Start(i)), Lhs(End(j))) => {
                    self.lhs_ck = false;
                    self.rhs_ck = true;
                    belongck!(i, j, BelongTo::Both)
                }

                (Rhs(Start(i)), Rhs(End(j))) => {
                    let belong_to = if self.lhs_ck {
                        BelongTo::Both(i..j)
                    } else {
                        BelongTo::Rhs(i..j)
                    };
                    self.rhs_ck = false;
                    return Some(belong_to);
                }

                (Rhs(End(_)), Lhs(Start(_))) => {
                    self.lhs_ck = true;
                    self.rhs_ck = false;
                    continue;
                }

                (Rhs(End(i)), Lhs(End(j))) => {
                    self.lhs_ck = false;
                    self.rhs_ck = false;
                    belongck!(i, j, BelongTo::Lhs)
                }

                (Rhs(End(i)), Rhs(Start(j))) => {
                    let belong_to = if self.lhs_ck {
                        BelongTo::Lhs(i..j)
                    } else {
                        // BelongTo::None(i..j)
                        continue;
                    };
                    self.rhs_ck = true;
                    return Some(belong_to);
                }

                _ => unreachable!(),
            }
        }
        None
    }
}

#[test]
fn test_overlap() {
    static LHS: &[Range] = &[3..=5, 10..=13, 18..=19, 100..=120];
    static RHS: &[Range] = &[2..=3, 6..=9, 12..=14, 17..=21, 200..=1000];
    static NULL: &[Range] = &[];
    static FULL: &[Range] = &[0..=u16::MAX];

    assert_eq!(
        Overlap::new(LHS, RHS).collect::<Vec<BelongTo<u32>>>(),
        vec![
            BelongTo::Rhs(2..3),
            BelongTo::Both(3..4),
            BelongTo::Lhs(4..6),
            BelongTo::Rhs(6..10),
            BelongTo::Lhs(10..12),
            BelongTo::Both(12..14),
            BelongTo::Rhs(14..15),
            BelongTo::Rhs(17..18),
            BelongTo::Both(18..20),
            BelongTo::Rhs(20..22),
            BelongTo::Lhs(100..121),
            BelongTo::Rhs(200..1001),
        ]
    );

    assert_eq!(
        Overlap::new(NULL, RHS).collect::<Vec<BelongTo<u32>>>(),
        vec![
            BelongTo::Rhs(2..4),
            BelongTo::Rhs(6..10),
            BelongTo::Rhs(12..15),
            BelongTo::Rhs(17..22),
            BelongTo::Rhs(200..1001),
        ]
    );

    assert_eq!(
        Overlap::new(LHS, NULL).collect::<Vec<BelongTo<u32>>>(),
        vec![
            BelongTo::Lhs(3..6),
            BelongTo::Lhs(10..14),
            BelongTo::Lhs(18..20),
            BelongTo::Lhs(100..121),
        ]
    );

    assert_eq!(
        Overlap::new(FULL, RHS).collect::<Vec<BelongTo<u32>>>(),
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
        Overlap::new(LHS, FULL).collect::<Vec<BelongTo<u32>>>(),
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

    let a1 = &[0..=1, 3..=5, 12..=16, 18..=19];
    let a2 = &[0..=0, 3..=8, 10..=13, 15..=15, 19..=19];

    assert_eq!(
        Overlap::new(a1, a2).collect::<Vec<BelongTo<u32>>>(),
        vec![
            BelongTo::Both(0..1),
            BelongTo::Lhs(1..2),
            BelongTo::Both(3..6),
            BelongTo::Rhs(6..9),
            BelongTo::Rhs(10..12),
            BelongTo::Both(12..14),
            BelongTo::Lhs(14..15),
            BelongTo::Both(15..16),
            BelongTo::Lhs(16..17),
            BelongTo::Lhs(18..19),
            BelongTo::Both(19..20),
        ]
    );

    assert_eq!(
        Overlap::new(a2, a1).collect::<Vec<BelongTo<u32>>>(),
        vec![
            BelongTo::Both(0..1),
            BelongTo::Rhs(1..2),
            BelongTo::Both(3..6),
            BelongTo::Lhs(6..9),
            BelongTo::Lhs(10..12),
            BelongTo::Both(12..14),
            BelongTo::Rhs(14..15),
            BelongTo::Both(15..16),
            BelongTo::Rhs(16..17),
            BelongTo::Rhs(18..19),
            BelongTo::Both(19..20),
        ]
    );
}

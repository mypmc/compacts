#![cfg_attr(feature = "cargo-clippy", allow(range_minus_one))]
#![cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]

use std::ops::{Range, RangeInclusive};
use std::{cmp, fmt, io, iter, u16};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use bits::{self, pair};
use io::{ReadFrom, WriteTo};
use super::{Arr64, Seq16};

#[derive(Clone, Default, PartialEq, Eq)]
pub(crate) struct Run16 {
    pub weight: u32,
    pub ranges: Vec<RangeInclusive<u16>>,
}
impl fmt::Debug for Run16 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Run16({:?})", self.weight)
    }
}

impl Run16 {
    pub fn new() -> Self {
        Run16::default()
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

    fn index_to_insert(&self, x: &u16) -> Option<usize> {
        self.search(x).err()
    }
    fn index_to_remove(&self, x: &u16) -> Option<usize> {
        self.search(x).ok()
    }

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
        inserted
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
}

impl From<Seq16> for Run16 {
    fn from(vec16: Seq16) -> Self {
        Run16::from(&vec16)
    }
}
impl<'a> From<&'a Seq16> for Run16 {
    fn from(vec16: &'a Seq16) -> Self {
        vec16.vector.iter().collect()
    }
}

impl From<Arr64> for Run16 {
    fn from(vec64: Arr64) -> Self {
        Run16::from(&vec64)
    }
}
impl<'a> From<&'a Arr64> for Run16 {
    fn from(arr64: &'a Arr64) -> Self {
        const WIDTH: u16 = bits::U64_BITSIZE as u16;
        let mut run = Run16::new();
        for (i, &bit) in arr64.boxarr.iter().enumerate().filter(|&(_, &v)| v != 0) {
            let mut bit = bit;
            for pos in 0..WIDTH {
                if bit & (1 << pos) != 0 {
                    let x = (i as u16 * WIDTH) + pos;
                    run.insert(x);
                    bit &= !(1 << pos);
                }
            }
        }
        run
    }
}

impl<'a> iter::FromIterator<u16> for Run16 {
    fn from_iter<I>(iterable: I) -> Self
    where
        I: IntoIterator<Item = u16>,
    {
        let mut run = Run16::new();
        for bit in iterable {
            run.insert(bit);
        }
        run
    }
}
impl<'a> iter::FromIterator<&'a u16> for Run16 {
    fn from_iter<I>(iterable: I) -> Self
    where
        I: IntoIterator<Item = &'a u16>,
    {
        let mut run = Run16::new();
        for bit in iterable {
            run.insert(*bit);
        }
        run
    }
}

impl<'a> From<&'a [RangeInclusive<u16>]> for Run16 {
    fn from(slice: &'a [RangeInclusive<u16>]) -> Self {
        let mut rle16 = Run16 {
            weight: 0,
            ranges: Vec::with_capacity(slice.len()),
        };
        for r in slice {
            let w = u32::from(r.end - r.start) + 1;
            rle16.weight += w;
            rle16.ranges.push(r.start..=r.end);
        }
        rle16
    }
}

macro_rules! twofold_filter {
    ( $this:expr, $that:expr, $fn:ident ) => {
        {
            let fold = TwoFold::new(&$this.ranges, &$that.ranges);
            let iter = fold.filter_map($fn);
            let (weight, ranges) = repair(iter);
            Run16 { weight, ranges }
        }
    }
}

impl<'a> bits::BitAndAssign<&'a Run16> for Run16 {
    fn bitand_assign(&mut self, rle16: &'a Run16) {
        *self = twofold_filter!(self, rle16, filter_and);
    }
}
impl<'a> bits::BitOrAssign<&'a Run16> for Run16 {
    fn bitor_assign(&mut self, rle16: &'a Run16) {
        *self = twofold_filter!(self, rle16, filter_or);
    }
}
impl<'a> bits::BitAndNotAssign<&'a Run16> for Run16 {
    fn bitandnot_assign(&mut self, rle16: &'a Run16) {
        *self = twofold_filter!(self, rle16, filter_and_not);
    }
}
impl<'a> bits::BitXorAssign<&'a Run16> for Run16 {
    fn bitxor_assign(&mut self, rle16: &'a Run16) {
        *self = twofold_filter!(self, rle16, filter_xor);
    }
}

struct TwoFold<'r, T> {
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
    Lhs(Range<T>),
    Rhs(Range<T>),
    Both(Range<T>),
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
    fn inner(&self) -> &Range<T> {
        match *self {
            BelongTo::Both(ref r) | BelongTo::Lhs(ref r) | BelongTo::Rhs(ref r) => r,
        }
    }
    fn range(&self) -> Range<T> {
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
fn merge<'a, 'b, 'r>(
    lhs: &'a [RangeInclusive<u16>],
    rhs: &'b [RangeInclusive<u16>],
) -> impl Iterator<Item = Boundary<u32>> + 'r
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

fn to_exclusive(range: &RangeInclusive<u16>) -> Range<u32> {
    let start = u32::from(range.start);
    let end = u32::from(range.end);
    start..(end + 1)
}

impl<'r> TwoFold<'r, u32> {
    // assume that each elements (range) has no overlap
    pub fn new<'a, 'b>(
        lhs: &'a [RangeInclusive<u16>],
        rhs: &'b [RangeInclusive<u16>],
    ) -> TwoFold<'r, u32>
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
        TwoFold {
            lhs_ck,
            rhs_ck,
            tuples,
        }
    }
}

fn filter_and(be: BelongTo<u32>) -> Option<Range<u32>> {
    match be {
        BelongTo::Both(_) => Some(be.range()),
        _ => None,
    }
}

fn filter_or(be: BelongTo<u32>) -> Option<Range<u32>> {
    Some(be.range())
}

fn filter_and_not(be: BelongTo<u32>) -> Option<Range<u32>> {
    match be {
        BelongTo::Lhs(_) => Some(be.range()),
        _ => None,
    }
}

fn filter_xor(be: BelongTo<u32>) -> Option<Range<u32>> {
    match be {
        BelongTo::Lhs(_) | BelongTo::Rhs(_) => Some(be.range()),
        _ => None,
    }
}

/// Repair broken ranges, and accumulate weight.
fn repair<I>(folded: I) -> (u32, Vec<RangeInclusive<u16>>)
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
            vec.push(start..=end);
            continue;
        }

        let i = vec.len();

        // leap should not happen
        assert!(vec[i - 1].end <= start);

        if start == (vec[i - 1].end + 1) {
            // merge into a previous range
            vec[i - 1] = vec[i - 1].start..=end;
        } else {
            vec.push(start..=end);
        }
    }
    (w, vec)
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

impl<'r> Iterator for TwoFold<'r, u32> {
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

// `Run16` is serialized as a 16-bit integer indicating the number of runs,
// followed by a pair of 16-bit values for each run.
// Runs are non-overlapping and sorted.
// Each pair of 16-bit values contains the starting index of the run
// followed by the length of the run minus 1.
// That is, we interleave values and lengths, so that if you have the values `[11,12,13,14,15]`,
// you store that as `11,4` where 4 means that beyond 11 itself,
// there are 4 contiguous values that follow.
//
// Example:
// `[(1,3),(20,0),(31,2)]` => `[1, 2, 3, 4, 20, 31, 32, 33]`

impl<W: io::Write> WriteTo<W> for Run16 {
    fn write_to(&self, w: &mut W) -> io::Result<()> {
        w.write_u16::<LittleEndian>(self.ranges.len() as u16)?;
        for rg in &self.ranges {
            w.write_u16::<LittleEndian>(rg.start)?;
            w.write_u16::<LittleEndian>(rg.end - rg.start)?;
        }
        Ok(())
    }
}

impl<R: io::Read> ReadFrom<R> for Run16 {
    // Resize automatically.
    fn read_from(&mut self, r: &mut R) -> io::Result<()> {
        let runs = r.read_u16::<LittleEndian>()?;
        self.weight = 0;
        self.ranges.resize(runs as usize, 0..=0);

        for rg in &mut self.ranges {
            let s = r.read_u16::<LittleEndian>()?;
            let o = r.read_u16::<LittleEndian>()?;
            *rg = s..=(s + o);
            self.weight += u32::from(o) + 1;
        }
        Ok(())
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

    fn test_identity(rle: &Run16) {
        let from_seq16 = Run16::from(Seq16::from(rle));
        let from_arr64 = Run16::from(Arr64::from(rle));
        assert_eq!(rle.weight, from_seq16.weight);
        assert_eq!(rle.ranges, from_seq16.ranges);
        assert_eq!(rle.weight, from_arr64.weight);
        assert_eq!(rle.ranges, from_arr64.ranges);
    }

    #[test]
    #[ignore]
    fn insert_remove() {
        let mut rle = Run16::new();

        test_identity(&rle);
        insert_all!(rle, 1, 3, 5, 4);
        assert_eq!(rle.weight, 4);
        assert_eq!(rle.ranges, &[1..=1, 3..=5]);

        test_identity(&rle);
        insert_all!(rle, 2, 8);
        assert_eq!(rle.weight, 6);
        assert_eq!(rle.ranges, &[1..=5, 8..=8]);

        test_identity(&rle);
        insert_all!(rle, 10, 7);
        assert_eq!(rle.weight, 8);
        assert_eq!(rle.ranges, &[1..=5, 7..=8, 10..=10]);

        test_identity(&rle);
        insert_all!(rle, 9, 6, 0);
        assert_eq!(rle.weight, 11);
        assert_eq!(rle.ranges, &[0..=10]);

        test_identity(&rle);
        insert_all!(rle, 65534, 65535);
        assert_eq!(rle.weight, 13);
        assert_eq!(rle.ranges, &[0..=10, 65534..=65535]);

        test_identity(&rle);
        remove_all!(rle, 65534, 65535);
        assert_eq!(rle.weight, 11);
        assert_eq!(rle.ranges, &[0..=10]);

        test_identity(&rle);
        remove_all!(rle, 0, 4);
        assert_eq!(rle.weight, 9);
        assert_eq!(rle.ranges, &[1..=3, 5..=10]);

        test_identity(&rle);
        remove_all!(rle, 7, 2);
        assert_eq!(rle.weight, 7);
        assert_eq!(rle.ranges, &[1..=1, 3..=3, 5..=6, 8..=10]);

        test_identity(&rle);
        remove_all!(rle, 3, 5);
        assert_eq!(rle.weight, 5);
        assert_eq!(rle.ranges, &[1..=1, 6..=6, 8..=10]);

        test_identity(&rle);
        remove_all!(rle, 1, 6);
        assert_eq!(rle.weight, 3);
        assert_eq!(rle.ranges, &[8..=10]);

        test_identity(&rle);
        remove_all!(rle, 10, 8);
        assert_eq!(rle.weight, 1);
        assert_eq!(rle.ranges, &[9..=9]);

        test_identity(&rle);
        remove_all!(rle, 9);
        assert_eq!(rle.weight, 0);
        assert_eq!(rle.ranges, &[]);
    }

    #[test]
    #[ignore]
    fn two_fold() {
        static LHS: &[RangeInclusive<u16>] = &[3..=5, 10..=13, 18..=19, 100..=120];
        static RHS: &[RangeInclusive<u16>] = &[2..=3, 6..=9, 12..=14, 17..=21, 200..=1000];
        static NULL: &[RangeInclusive<u16>] = &[];
        static FULL: &[RangeInclusive<u16>] = &[0..=u16::MAX];

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
                BelongTo::Rhs(17..18),
                BelongTo::Both(18..20),
                BelongTo::Rhs(20..22),
                BelongTo::Lhs(100..121),
                BelongTo::Rhs(200..1001),
            ]
        );

        assert_eq!(
            TwoFold::new(NULL, RHS).collect::<Vec<BelongTo<u32>>>(),
            vec![
                BelongTo::Rhs(2..4),
                BelongTo::Rhs(6..10),
                BelongTo::Rhs(12..15),
                BelongTo::Rhs(17..22),
                BelongTo::Rhs(200..1001),
            ]
        );

        assert_eq!(
            TwoFold::new(LHS, NULL).collect::<Vec<BelongTo<u32>>>(),
            vec![
                BelongTo::Lhs(3..6),
                BelongTo::Lhs(10..14),
                BelongTo::Lhs(18..20),
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

        let a1 = &[0..=1, 3..=5, 12..=16, 18..=19];
        let a2 = &[0..=0, 3..=8, 10..=13, 15..=15, 19..=19];

        assert_eq!(
            TwoFold::new(a1, a2).collect::<Vec<BelongTo<u32>>>(),
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
            TwoFold::new(a2, a1).collect::<Vec<BelongTo<u32>>>(),
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
}

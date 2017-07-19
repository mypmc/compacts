use std::ops::{Range, RangeInclusive};
use std::{cmp, u16};
use itertools;

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
    let start = range.start as u32;
    let end = range.end as u32;
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

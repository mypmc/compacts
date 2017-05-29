use std::collections::VecDeque;
use std::ops::{Range, RangeInclusive};
use std::cmp;
use std::u16;
use itertools;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BelongTo<T: ::UnsignedInt> {
    None(Range<T>),
    Lhs(Range<T>),
    Rhs(Range<T>),
    Both(Range<T>),
}
impl<T: ::UnsignedInt> BelongTo<T> {
    pub fn range(&self) -> Range<T> {
        match *self {
            BelongTo::None(ref r) => r.start..r.end,
            BelongTo::Lhs(ref r) => r.start..r.end,
            BelongTo::Rhs(ref r) => r.start..r.end,
            BelongTo::Both(ref r) => r.start..r.end,
        }
    }
}

pub struct Folding<'r, T: ::UnsignedInt> {
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

#[derive(Debug, PartialEq, Eq, PartialOrd)]
enum Side {
    Lhs,
    Rhs,
}

// half open
#[derive(Debug, Clone, PartialEq, Eq)]
enum Boundary<T: ::UnsignedInt> {
    Lhs(State<T>),
    Rhs(State<T>),
}
use self::Boundary::*;

impl<T: ::UnsignedInt> Boundary<T> {
    fn value(&self) -> T {
        match *self {
            Lhs(Open(i)) => i,
            Lhs(Close(i)) => i,
            Rhs(Open(i)) => i,
            Rhs(Close(i)) => i,
        }
    }
}
impl<'a, T: ::UnsignedInt> PartialOrd<Boundary<T>> for Boundary<T> {
    fn partial_cmp(&self, rhs: &Boundary<T>) -> Option<cmp::Ordering> {
        Some(self.value().cmp(&rhs.value()))
    }
}
impl<'a, T: ::UnsignedInt> Ord for Boundary<T> {
    fn cmp(&self, rhs: &Boundary<T>) -> cmp::Ordering {
        self.value().cmp(&rhs.value())
    }
}

impl<'r, T: ::UnsignedInt> Folding<'r, T> {
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

impl<'r> Folding<'r, u32> {
    // assume that each elements (range) has no overlap
    pub fn new<'a, 'b>(l: &'a [RangeInclusive<u16>],
                       r: &'b [RangeInclusive<u16>])
                       -> Folding<'r, u32>
        where 'a: 'r,
              'b: 'r
    {
        use itertools::Itertools;

        let lhs = None;
        let rhs = None;
        let window = Box::new(merge(l, r)
                                  .tuple_windows()
                                  .filter(|&(ref i, ref j)| i.value() != j.value()));
        Folding { lhs, rhs, window }
    }

    pub fn intersection(self) -> impl Iterator<Item = Range<u32>> + 'r {
        self.filter(|belong| match belong {
                        &BelongTo::Both(_) => true,
                        _ => false,
                    })
            .map(|b| b.range())
    }

    pub fn union(self) -> impl Iterator<Item = Range<u32>> + 'r {
        self.filter(|be| match be {
                        &BelongTo::None(_) => false,
                        _ => true,
                    })
            .map(|be| be.range())
    }

    pub fn difference(self) -> impl Iterator<Item = Range<u32>> + 'r {
        self.filter(|be| match be {
                        &BelongTo::Lhs(_) => true,
                        _ => false,
                    })
            .map(|be| be.range())
    }

    pub fn symmetric_difference(self) -> impl Iterator<Item = Range<u32>> + 'r {
        self.filter(|be| match be {
                        &BelongTo::Lhs(_) |
                        &BelongTo::Rhs(_) => true,
                        _ => false,
                    })
            .map(|be| be.range())
    }
}

/// Repair broken ranges, and accumulate weight.
pub fn repair<I>(folded: I) -> (u32, Vec<RangeInclusive<u16>>)
    where I: IntoIterator<Item = Range<u32>>
{
    let mut vec = Vec::new();
    let mut w = 0;
    for curr in folded {
        // doesn't allow value like (3..2)
        assert!(curr.start < curr.end);

        w += curr.end - curr.start;

        let start = curr.start as u16;
        let end = (curr.end - 1) as u16;

        if vec.len() == 0 {
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


impl<'r> Iterator for Folding<'r, u32> {
    type Item = BelongTo<u32>;
    fn next(&mut self) -> Option<BelongTo<u32>> {
        self.window
            .next()
            .map(|next| match next {
                     (Lhs(Open(i)), Lhs(Close(j))) => {
                         let belong_to = if self.rhs_is_open() {
                             BelongTo::Both(i..j)
                         } else {
                             BelongTo::Lhs(i..j)
                         };
                         self.lhs_swap(Close(i));
                         belong_to
                     }
                     (Lhs(Open(i)), Rhs(Open(j))) => {
                         self.lhs_swap(Open(i));
                         self.rhs_swap(Open(j));
                         BelongTo::Lhs(i..j)
                     }
                     (Lhs(Open(i)), Rhs(Close(j))) => {
                         self.lhs_swap(Open(i));
                         self.rhs_swap(Close(j));
                         BelongTo::Both(i..j)
                     }
                     (Lhs(Close(i)), Lhs(Open(j))) => {
                         let belong_to = if self.rhs_is_open() {
                             BelongTo::Rhs(i..j)
                         } else {
                             BelongTo::None(i..j)
                         };
                         self.lhs_swap(Close(i));
                         self.rhs_swap(Open(j));
                         belong_to
                     }
                     (Lhs(Close(i)), Rhs(Open(j))) => {
                         self.lhs_swap(Close(i));
                         self.rhs_swap(Open(j));
                         BelongTo::None(i..j)
                     }
                     (Lhs(Close(i)), Rhs(Close(j))) => {
                         self.lhs_swap(Close(i));
                         self.rhs_swap(Close(j));
                         BelongTo::Rhs(i..j)
                     }
                     (Rhs(Open(i)), Lhs(Open(j))) => {
                         self.lhs_swap(Open(j));
                         self.rhs_swap(Open(i));
                         BelongTo::Rhs(i..j)
                     }
                     (Rhs(Open(i)), Lhs(Close(j))) => {
                         self.lhs_swap(Close(j));
                         self.rhs_swap(Open(i));
                         BelongTo::Both(i..j)
                     }
                     (Rhs(Open(i)), Rhs(Close(j))) => {
                         let belong_to = if self.lhs_is_open() {
                             BelongTo::Both(i..j)
                         } else {
                             BelongTo::Rhs(i..j)
                         };
                         self.rhs_swap(Close(j));
                         belong_to
                     }
                     (Rhs(Close(i)), Lhs(Open(j))) => {
                         self.lhs_swap(Open(j));
                         self.rhs_swap(Close(i));
                         BelongTo::None(i..j)
                     }
                     (Rhs(Close(i)), Lhs(Close(j))) => {
                         self.lhs_swap(Close(j));
                         self.rhs_swap(Close(i));
                         BelongTo::Lhs(i..j)
                     }
                     (Rhs(Close(i)), Rhs(Open(j))) => {
                         let belong_to = if self.lhs_is_open() {
                             BelongTo::Lhs(i..j)
                         } else {
                             BelongTo::None(i..j)
                         };
                         self.rhs_swap(Open(j));
                         belong_to
                     }
                     _ => unreachable!(),
                 })
    }
}

// assume that each elements (range) has no overlap
fn merge<'a, 'b, 'r>(lhs: &'a [RangeInclusive<u16>],
                     rhs: &'b [RangeInclusive<u16>])
                     -> impl Iterator<Item = Boundary<u32>> + 'r
    where 'a: 'r,
          'b: 'r
{
    let lhs_iter = lhs.iter()
        .map(to_halfopen)
        .flat_map(|range| enqueue(range, Side::Lhs));

    let rhs_iter = rhs.iter()
        .map(to_halfopen)
        .flat_map(|range| enqueue(range, Side::Rhs));

    itertools::merge(lhs_iter, rhs_iter)
}

fn enqueue<T>(range: Range<T>, side: Side) -> VecDeque<Boundary<T>>
    where T: ::UnsignedInt
{
    let mut queue = VecDeque::new();
    match side {
        Side::Lhs => {
            queue.push_back(Boundary::Lhs(Open(range.start)));
            queue.push_back(Boundary::Lhs(Close(range.end)));
        }
        Side::Rhs => {
            queue.push_back(Boundary::Rhs(Open(range.start)));
            queue.push_back(Boundary::Rhs(Close(range.end)));
        }
    };
    queue
}

fn to_halfopen(range: &RangeInclusive<u16>) -> Range<u32> {
    let start = range.start as u32;
    let end = range.end as u32;
    (start..(end + 1))
}

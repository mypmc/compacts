use std::cmp::Ordering;

use crate::bit::ops::Finite;

use super::{super::MapEncode, MergeBy, Tuples};

pub struct Members<'r> {
    finished: bool,
    last_val: Option<Member>,
    members: std::iter::Peekable<IntoMembers<'r>>,
}

struct IntoMembers<'r> {
    lhs_is_open: bool, // remember whether last lhs is open
    rhs_is_open: bool, // remember whether last rhs is open
    tuples: Box<Iterator<Item = (Merged, Merged)> + 'r>,
}

#[derive(Clone, Debug)]
enum Pair {
    Bra(u32), // [
    Ket(u32), // )
}

#[derive(Clone, Debug)]
enum Merged {
    Lhs(Pair),
    Rhs(Pair),
}

impl Pair {
    fn value(&self) -> u32 {
        match self {
            Pair::Bra(i) | Pair::Ket(i) => *i,
        }
    }
}

impl PartialEq<Pair> for Pair {
    fn eq(&self, rhs: &Pair) -> bool {
        self.value().eq(&rhs.value())
    }
}
impl Eq for Pair {}

impl PartialOrd<Pair> for Pair {
    fn partial_cmp(&self, rhs: &Pair) -> Option<Ordering> {
        self.value().partial_cmp(&rhs.value())
    }
}
impl Ord for Pair {
    fn cmp(&self, rhs: &Pair) -> Ordering {
        self.value().cmp(&rhs.value())
    }
}

// assume that each elements (range) has no overlap
fn merged<'a, 'b, 'r, L, R>(
    lhs: L,
    rhs: R,
) -> MergeBy<
    impl Iterator<Item = Merged> + 'a,
    impl Iterator<Item = Merged> + 'b,
    impl Fn(&Merged, &Merged) -> Ordering,
>
where
    L: Iterator<Item = &'a std::ops::RangeInclusive<u16>> + 'a,
    R: Iterator<Item = &'b std::ops::RangeInclusive<u16>> + 'b,
    'a: 'r,
    'b: 'r,
{
    let lhs = lhs.flat_map(|r| {
        vec![
            Merged::Lhs(Pair::Bra(u32::from(*r.start()))),
            Merged::Lhs(Pair::Ket(u32::from(*r.end()) + 1)),
        ]
    });
    let rhs = rhs.flat_map(|r| {
        vec![
            Merged::Rhs(Pair::Bra(u32::from(*r.start()))),
            Merged::Rhs(Pair::Ket(u32::from(*r.end()) + 1)),
        ]
    });
    MergeBy::merge_by(lhs, rhs, |a, b| match (a, b) {
        (Merged::Lhs(a), Merged::Lhs(b)) => a.cmp(b),
        (Merged::Lhs(a), Merged::Rhs(b)) => a.cmp(b),
        (Merged::Rhs(a), Merged::Lhs(b)) => a.cmp(b),
        (Merged::Rhs(a), Merged::Rhs(b)) => a.cmp(b),
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Member {
    Lhs(std::ops::Range<u32>), // L and not R
    Rhs(std::ops::Range<u32>), // R and not L
    And(std::ops::Range<u32>), // L and R
    Not(std::ops::Range<u32>), // not (L or R)
}
impl Member {
    fn range(&self) -> &std::ops::Range<u32> {
        match self {
            Member::Lhs(r) | Member::Rhs(r) | Member::And(r) | Member::Not(r) => r,
        }
    }
}

impl<'r> Members<'r> {
    pub fn new<'a, 'b>(
        lhs: impl IntoIterator<Item = &'a std::ops::RangeInclusive<u16>> + 'a,
        rhs: impl IntoIterator<Item = &'b std::ops::RangeInclusive<u16>> + 'b,
    ) -> Self
    where
        'a: 'r,
        'b: 'r,
    {
        let finished = false;
        let last_val = None;
        let members = IntoMembers::new(lhs, rhs).peekable();
        Members {
            finished,
            last_val,
            members,
        }
    }

    pub fn filter_and(self) -> impl Iterator<Item = std::ops::Range<u32>> + 'r {
        self.filter_map(|member| match member {
            Member::And(range) => Some(range),
            _ => None,
        })
    }

    pub fn filter_or(self) -> impl Iterator<Item = std::ops::Range<u32>> + 'r {
        self.filter_map(|member| match member {
            Member::Lhs(range) => Some(range),
            Member::Rhs(range) => Some(range),
            Member::And(range) => Some(range),
            _ => None,
        })
    }

    pub fn filter_xor(self) -> impl Iterator<Item = std::ops::Range<u32>> + 'r {
        self.filter_map(|member| match member {
            Member::Lhs(range) | Member::Rhs(range) => Some(range),
            _ => None,
        })
    }

    pub fn filter_not(self) -> impl Iterator<Item = std::ops::Range<u32>> + 'r {
        self.filter_map(|member| match member {
            Member::Not(range) => Some(range),
            _ => None,
        })
    }
}

impl<'r> IntoMembers<'r> {
    fn new<'a, 'b>(
        lhs: impl IntoIterator<Item = &'a std::ops::RangeInclusive<u16>> + 'a,
        rhs: impl IntoIterator<Item = &'b std::ops::RangeInclusive<u16>> + 'b,
    ) -> Self
    where
        'a: 'r,
        'b: 'r,
    {
        let lhs_is_open = false;
        let rhs_is_open = false;
        let tuples = {
            let merged = merged(lhs.into_iter(), rhs.into_iter());
            Box::new(Tuples::tuples(merged))
        };
        IntoMembers {
            lhs_is_open,
            rhs_is_open,
            tuples,
        }
    }
}

impl<'r> Iterator for Members<'r> {
    type Item = Member;
    fn next(&mut self) -> Option<Member> {
        if self.finished {
            return None;
        };
        let peek = self.members.peek();
        match (self.last_val.clone(), peek) {
            (None, Some(head)) => {
                let range = head.range();
                if range.start == 0 {
                    self.last_val = self.members.next();
                    self.last_val.clone()
                } else {
                    self.last_val = Some(Member::Not(0..range.start));
                    self.last_val.clone()
                }
            }

            (Some(_), Some(next)) => {
                let out = Some(next.clone());
                self.last_val = self.members.next();
                out
            }

            (Some(last), None) => {
                self.finished = true;
                let range = last.range();
                if range.end < MapEncode::BITS as u32 {
                    Some(Member::Not(range.end..MapEncode::BITS as u32))
                } else {
                    None
                }
            }

            // iterator is empty
            (None, None) => {
                self.finished = true;
                Some(Member::Not(0..MapEncode::BITS as u32))
            }
        }
    }
}

macro_rules! guard {
    ( $i:expr, $j:expr ) => {
        if $i == $j {
            continue;
        }
    };
    ( $i:expr, $j:expr, $member:expr ) => {
        if $i == $j {
            continue;
        } else {
            return Some($member($i..$j));
        }
    };
}

impl<'r> Iterator for IntoMembers<'r> {
    type Item = Member;
    fn next(&mut self) -> Option<Member> {
        use self::{Merged::*, Pair::*};

        while let Some(next) = self.tuples.next() {
            match next {
                (Lhs(Bra(i)), Lhs(Ket(j))) => {
                    self.lhs_is_open = false;
                    guard!(i, j);
                    return Some(if self.rhs_is_open {
                        Member::And(i..j)
                    } else {
                        Member::Lhs(i..j)
                    });
                }

                (Lhs(Ket(i)), Lhs(Bra(j))) => {
                    self.lhs_is_open = true;
                    guard!(i, j);
                    return Some(if self.rhs_is_open {
                        Member::Rhs(i..j)
                    } else {
                        Member::Not(i..j)
                    });
                }

                (Rhs(Bra(i)), Rhs(Ket(j))) => {
                    self.rhs_is_open = false;
                    guard!(i, j);
                    let member = if self.lhs_is_open {
                        Member::And(i..j)
                    } else {
                        Member::Rhs(i..j)
                    };
                    return Some(member);
                }

                (Rhs(Ket(i)), Rhs(Bra(j))) => {
                    self.rhs_is_open = true;
                    guard!(i, j);
                    return Some(if self.lhs_is_open {
                        Member::Lhs(i..j)
                    } else {
                        Member::Not(i..j)
                    });
                }

                (Lhs(Bra(i)), Rhs(Bra(j))) => {
                    self.lhs_is_open = true;
                    self.rhs_is_open = true;
                    guard!(i, j, Member::Lhs)
                }

                (Lhs(Bra(i)), Rhs(Ket(j))) => {
                    self.lhs_is_open = true;
                    self.rhs_is_open = false;
                    guard!(i, j, Member::And)
                }

                (Lhs(Ket(i)), Rhs(Bra(j))) => {
                    self.lhs_is_open = false;
                    self.rhs_is_open = true;
                    guard!(i, j, Member::Not)
                }

                (Lhs(Ket(i)), Rhs(Ket(j))) => {
                    self.lhs_is_open = false;
                    self.rhs_is_open = false;
                    guard!(i, j, Member::Rhs)
                }

                (Rhs(Bra(i)), Lhs(Bra(j))) => {
                    self.lhs_is_open = true;
                    self.rhs_is_open = true;
                    guard!(i, j, Member::Rhs)
                }

                (Rhs(Bra(i)), Lhs(Ket(j))) => {
                    self.lhs_is_open = false;
                    self.rhs_is_open = true;
                    guard!(i, j, Member::And)
                }

                (Rhs(Ket(i)), Lhs(Bra(j))) => {
                    self.lhs_is_open = true;
                    self.rhs_is_open = false;
                    guard!(i, j, Member::Not)
                }

                (Rhs(Ket(i)), Lhs(Ket(j))) => {
                    self.lhs_is_open = false;
                    self.rhs_is_open = false;
                    guard!(i, j, Member::Lhs)
                }

                _ => unreachable!(),
            }
        }
        None
    }
}

#[cfg(test)]
#[test]
fn test_into_members() {
    type Range = std::ops::RangeInclusive<u16>;

    let a = &[];
    let b = &[];
    assert_eq!(
        Members::new(a.iter(), b.iter()).collect::<Vec<Member>>(),
        vec![Member::Not(0..65536)]
    );

    let a: &[Range] = &[3..=5, 6..=10];
    let b: &[Range] = &[2..=3];
    assert_eq!(
        Members::new(a.iter(), b.iter()).collect::<Vec<Member>>(),
        vec![
            Member::Not(0..2),
            Member::Rhs(2..3),
            Member::And(3..4),
            Member::Lhs(4..6),
            Member::Lhs(6..11),
            Member::Not(11..65536),
        ]
    );

    let a: &[Range] = &[3..=5, 6..=10];
    let b: &[Range] = &[2..=3, 6..=6];
    assert_eq!(
        Members::new(a.iter(), b.iter()).collect::<Vec<Member>>(),
        vec![
            Member::Not(0..2),
            Member::Rhs(2..3),
            Member::And(3..4),
            Member::Lhs(4..6),
            Member::And(6..7),
            Member::Lhs(7..11),
            Member::Not(11..65536),
        ]
    );

    let a: &[Range] = &[3..=5, 6..=10, 18..=18];
    let b: &[Range] = &[2..=3, 6..=6];
    assert_eq!(
        Members::new(a.iter(), b.iter()).collect::<Vec<Member>>(),
        vec![
            Member::Not(0..2),
            Member::Rhs(2..3),
            Member::And(3..4),
            Member::Lhs(4..6),
            Member::And(6..7),
            Member::Lhs(7..11),
            Member::Not(11..18),
            Member::Lhs(18..19),
            Member::Not(19..65536),
        ]
    );

    let a: &[Range] = &[3..=5, 10..=13, 18..=19, 100..=120];
    let b: &[Range] = &[2..=3, 6..=9, 12..=14, 17..=21, 200..=1000];
    assert_eq!(
        Members::new(a.iter(), b.iter()).collect::<Vec<Member>>(),
        vec![
            Member::Not(0..2),
            Member::Rhs(2..3),
            Member::And(3..4),
            Member::Lhs(4..6),
            Member::Rhs(6..10),
            Member::Lhs(10..12),
            Member::And(12..14),
            Member::Rhs(14..15),
            Member::Not(15..17),
            Member::Rhs(17..18),
            Member::And(18..20),
            Member::Rhs(20..22),
            Member::Not(22..100),
            Member::Lhs(100..121),
            Member::Not(121..200),
            Member::Rhs(200..1001),
            Member::Not(1001..65536),
        ]
    );

    let a: &[Range] = &[];
    let b: &[Range] = &[2..=3, 6..=9, 12..=14, 17..=21, 200..=1000];
    assert_eq!(
        Members::new(a.iter(), b.iter()).collect::<Vec<Member>>(),
        vec![
            Member::Not(0..2),
            Member::Rhs(2..4),
            Member::Not(4..6),
            Member::Rhs(6..10),
            Member::Not(10..12),
            Member::Rhs(12..15),
            Member::Not(15..17),
            Member::Rhs(17..22),
            Member::Not(22..200),
            Member::Rhs(200..1001),
            Member::Not(1001..65536),
        ]
    );

    let a: &[Range] = &[3..=5, 10..=13, 18..=19, 100..=120];
    let b: &[Range] = &[];
    assert_eq!(
        Members::new(a.iter(), b.iter()).collect::<Vec<Member>>(),
        vec![
            Member::Not(0..3),
            Member::Lhs(3..6),
            Member::Not(6..10),
            Member::Lhs(10..14),
            Member::Not(14..18),
            Member::Lhs(18..20),
            Member::Not(20..100),
            Member::Lhs(100..121),
            Member::Not(121..65536),
        ]
    );

    let a: &[Range] = &[0..=!0];
    let b: &[Range] = &[2..=3, 6..=9, 12..=14, 17..=21, 200..=1000];
    assert_eq!(
        Members::new(a.iter(), b.iter()).collect::<Vec<Member>>(),
        vec![
            Member::Lhs(0..2),
            Member::And(2..4),
            Member::Lhs(4..6),
            Member::And(6..10),
            Member::Lhs(10..12),
            Member::And(12..15),
            Member::Lhs(15..17),
            Member::And(17..22),
            Member::Lhs(22..200),
            Member::And(200..1001),
            Member::Lhs(1001..65536),
        ]
    );

    let a = &[0..=1, 3..=5, 12..=16, 18..=19];
    let b = &[0..=0, 3..=8, 10..=13, 15..=15, 19..=19];
    assert_eq!(
        Members::new(a.iter(), b.iter()).collect::<Vec<Member>>(),
        vec![
            Member::And(0..1),
            Member::Lhs(1..2),
            Member::Not(2..3),
            Member::And(3..6),
            Member::Rhs(6..9),
            Member::Not(9..10),
            Member::Rhs(10..12),
            Member::And(12..14),
            Member::Lhs(14..15),
            Member::And(15..16),
            Member::Lhs(16..17),
            Member::Not(17..18),
            Member::Lhs(18..19),
            Member::And(19..20),
            Member::Not(20..65536),
        ]
    );
}

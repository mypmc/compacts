/// Boxed slice of words.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Map<B>(Option<Box<[B]>>);

impl Encode {
    const BOXED_SLICE_LEN: usize = BLOCK_SIZE / u64::BITS as usize;
    const BINARY_HEAP_MAX: usize = BLOCK_SIZE / u16::BITS as usize;

    fn into_boxed_slice(self) -> Map<u64> {
        match self {
            Encode::Map(map) => map,
            Encode::Bin(bin) => Map::<u64>::from(&bin),
            Encode::Run(run) => Map::<u64>::from(&run),
        }
    }

    fn as_boxed_slice(&mut self) {
        match self {
            Encode::Map(_) => {}
            Encode::Bin(ref bin) => {
                *self = Encode::Map(Map::<u64>::from(bin));
            }
            Encode::Run(ref run) => {
                *self = Encode::Map(Map::<u64>::from(run));
            }
        }
    }
}

impl From<Map<u64>> for Encode {
    fn from(data: Map<u64>) -> Self {
        Encode::Map(data)
    }
}
impl From<Bin> for Encode {
    fn from(data: Bin) -> Self {
        Encode::Bin(data)
    }
}
impl From<Run> for Encode {
    fn from(data: Run) -> Self {
        Encode::Run(data)
    }
}

// impl<T: Uint> From<&'_ Map<T>> for Bin {
//     fn from(vec: &Map<T>) -> Self {
//         let mut bin = Bin::with_capacity(Encode::BINARY_HEAP_MAX);
//         for (i, word) in vec.iter().enumerate().filter(|&(_, v)| v != T::ZERO) {
//             let offset = crate::uint::cast::<usize, u64>(i) * T::SIZE;
//             for j in 0..word.size() {
//                 if word.access(j) {
//                     bin.insert(offset + j);
//                 }
//             }
//         }
//         bin
//     }
// }

macro_rules! delegate {
    ( $this:ident, $method:ident $(, $args:expr )* ) => {
        {
            match $this {
                Encode::Map(data) => data.$method( $( $args ),* ),
                Encode::Bin(data) => data.$method( $( $args ),* ),
                Encode::Run(data) => data.$method( $( $args ),* ),
            }
        }
    };
}

impl FiniteBits for Array {
    const BITS: u64 = BLOCK_SIZE as u64;
    fn empty() -> Self {
        Self::default()
    }
}

impl FiniteBits for RoaringBlock {
    const BITS: u64 = BLOCK_SIZE as u64;
    fn empty() -> Self {
        Self::default()
    }
}

impl FiniteBits for Encode {
    const BITS: u64 = BLOCK_SIZE as u64;
    fn empty() -> Self {
        Self::default()
    }
}

impl Access for Array {
    #[inline]
    fn access(&self, i: u64) -> bool {
        assert!(i < Self::BITS);
        self.0.access(i)
    }
}

impl Access for RoaringBlock {
    #[inline]
    fn access(&self, i: u64) -> bool {
        assert!(i < Self::BITS);
        self.0.access(i)
    }
}

impl Access for Encode {
    #[inline]
    fn access(&self, i: u64) -> bool {
        delegate!(self, access, i)
    }
}

impl Count for Array {
    #[inline]
    fn bits(&self) -> u64 {
        Self::BITS
    }
    #[inline]
    fn count1(&self) -> u64 {
        self.0.count1()
    }
}

impl Count for RoaringBlock {
    #[inline]
    fn bits(&self) -> u64 {
        Self::BITS
    }
    #[inline]
    fn count1(&self) -> u64 {
        self.0.count1()
    }
}

impl Count for Encode {
    #[inline]
    fn bits(&self) -> u64 {
        Self::BITS
    }
    #[inline]
    fn count1(&self) -> u64 {
        delegate!(self, count1)
    }
}

impl Rank for Array {
    #[inline]
    fn rank1(&self, i: u64) -> u64 {
        self.0.rank1(i)
    }
}
impl Rank for RoaringBlock {
    #[inline]
    fn rank1(&self, i: u64) -> u64 {
        self.0.rank1(i)
    }
}
impl Rank for Encode {
    #[inline]
    fn rank1(&self, i: u64) -> u64 {
        delegate!(self, rank1, i)
    }
}

impl Select1 for Array {
    #[inline]
    fn select1(&self, c: u64) -> Option<u64> {
        self.0.select1(c)
    }
}
impl Select0 for Array {
    #[inline]
    fn select0(&self, c: u64) -> Option<u64> {
        self.0.select0(c)
    }
}

impl Select1 for RoaringBlock {
    #[inline]
    fn select1(&self, c: u64) -> Option<u64> {
        self.0.select1(c)
    }
}
impl Select0 for RoaringBlock {
    #[inline]
    fn select0(&self, c: u64) -> Option<u64> {
        self.0.select0(c)
    }
}

impl Select1 for Encode {
    #[inline]
    fn select1(&self, c: u64) -> Option<u64> {
        delegate!(self, select1, c)
    }
}
impl Select0 for Encode {
    #[inline]
    fn select0(&self, c: u64) -> Option<u64> {
        delegate!(self, select0, c)
    }
}

impl Assign<u64> for Array {
    type Output = ();
    #[inline]
    fn set1(&mut self, index: u64) -> Self::Output {
        self.0.set1(index)
    }
    #[inline]
    fn set0(&mut self, index: u64) -> Self::Output {
        self.0.set0(index)
    }
}
impl Assign<std::ops::Range<u64>> for Array {
    type Output = ();
    #[inline]
    fn set1(&mut self, index: std::ops::Range<u64>) -> Self::Output {
        self.0.set1(index);
    }
    #[inline]
    fn set0(&mut self, index: std::ops::Range<u64>) -> Self::Output {
        self.0.set0(index);
    }
}

impl Assign<u64> for RoaringBlock {
    type Output = ();
    #[inline]
    fn set1(&mut self, i: u64) -> Self::Output {
        self.0.set1(i)
    }
    #[inline]
    fn set0(&mut self, i: u64) -> Self::Output {
        self.0.set0(i)
    }
}

impl Assign<u64> for Encode {
    type Output = ();
    fn set1(&mut self, i: u64) -> Self::Output {
        assert!(i < Self::BITS);
        match self {
            Encode::Map(map) => map.set1(i),
            Encode::Bin(bin) => {
                if !bin.access(i) {
                    bin.set1(i);
                    if bin.len() >= Encode::BINARY_HEAP_MAX {
                        *self = Encode::Map(Map::<u64>::from(&*bin));
                    }
                }
            }
            Encode::Run(run) => run.set1(i),
        }
    }
    fn set0(&mut self, i: u64) -> Self::Output {
        delegate!(self, set0, i)
    }
}

impl Assign<std::ops::Range<u64>> for Encode {
    type Output = ();
    fn set1(&mut self, index: std::ops::Range<u64>) {
        delegate!(self, set1, index)
    }
    fn set0(&mut self, index: std::ops::Range<u64>) {
        delegate!(self, set0, index)
    }
}

impl std::ops::BitAndAssign<&'_ Encode> for Encode {
    fn bitand_assign(&mut self, encode: &Encode) {
        match (self, encode) {
            (Encode::Map(map1), Encode::Map(map2)) => map1.bitand_assign(map2),
            (Encode::Run(run1), Encode::Run(run2)) => run1.bitand_assign(run2),

            (Encode::Bin(bin1), Encode::Bin(bin2)) => bin1.bitand_assign(bin2),

            // FIXME: can be more efficient
            (Encode::Map(map), Encode::Bin(bin)) => map.bitand_assign(&Map::from(bin)),
            (Encode::Map(map), Encode::Run(run)) => map.bitand_assign(&Map::from(run)),

            (Encode::Bin(bin), Encode::Map(map)) => bin.0.retain(|&x| map.access(u64::from(x))),

            (this @ Encode::Run(_), that @ Encode::Map(_)) => {
                this.as_boxed_slice();
                this.bitand_assign(that);
            }

            // FIXME: use Members
            (this @ Encode::Bin(_), that @ Encode::Run(_)) => {
                this.as_boxed_slice();
                this.bitand_assign(that);
            }
            (this @ Encode::Run(_), that @ Encode::Bin(_)) => {
                this.as_boxed_slice();
                this.bitand_assign(that);
            }
        }
    }
}

impl std::ops::BitOrAssign<&'_ Encode> for Encode {
    fn bitor_assign(&mut self, encode: &Encode) {
        match (self, encode) {
            (Encode::Map(map1), Encode::Map(map2)) => map1.bitor_assign(map2),
            (Encode::Run(run1), Encode::Run(run2)) => run1.bitor_assign(run2),

            // (Encode::Bin(bin1), Encode::Bin(bin2)) => bin1.bitor_assign(bin2),
            (Encode::Map(map), Encode::Bin(bin)) => {
                for &x in bin {
                    map.set1(u64::from(x));
                }
            }

            (Encode::Map(map), Encode::Run(run)) => {
                for r in run.0.iter() {
                    let i = u64::from(*r.start());;
                    let len = u64::from(*r.end()) - i + 1;
                    map.set1(i..len);
                }
            }

            (this @ Encode::Bin(_), that @ Encode::Map(_)) => {
                this.as_boxed_slice();
                this.bitor_assign(that);
            }

            (this @ Encode::Run(_), that @ Encode::Map(_)) => {
                this.as_boxed_slice();
                this.bitor_assign(that);
            }

            (this @ Encode::Bin(_), that @ Encode::Bin(_)) => {
                this.as_boxed_slice();
                this.bitor_assign(that);
            }

            // FIXME: use Members
            (this @ Encode::Bin(_), that @ Encode::Run(_)) => {
                this.as_boxed_slice();
                this.bitor_assign(that);
            }
            (this @ Encode::Run(_), that @ Encode::Bin(_)) => {
                this.as_boxed_slice();
                this.bitor_assign(that);
            }
        }
    }
}

impl std::ops::BitXorAssign<&'_ Encode> for Encode {
    fn bitxor_assign(&mut self, encode: &Encode) {
        match (self, encode) {
            (Encode::Map(map1), Encode::Map(map2)) => map1.bitxor_assign(map2),
            (Encode::Run(run1), Encode::Run(run2)) => run1.bitxor_assign(run2),

            // (Encode::Bin(bin1), Encode::Bin(bin2)) => bin1.bitxor_assign(bin2),
            (Encode::Map(map), Encode::Bin(bin)) => {
                for &x in bin {
                    let i = u64::from(x);
                    if map.access(i) {
                        map.set0(i);
                    } else {
                        map.set1(i);
                    }
                }
            }

            (Encode::Map(map), Encode::Run(run)) => map.bitxor_assign(&Map::from(run)),

            (this @ Encode::Bin(_), that @ Encode::Map(_)) => {
                this.as_boxed_slice();
                this.bitxor_assign(that);
            }

            (this @ Encode::Run(_), that @ Encode::Map(_)) => {
                this.as_boxed_slice();
                this.bitxor_assign(that);
            }

            (this @ Encode::Bin(_), that @ Encode::Bin(_)) => {
                this.as_boxed_slice();
                this.bitxor_assign(that);
            }

            // FIXME: use Members
            (this @ Encode::Bin(_), that @ Encode::Run(_)) => {
                this.as_boxed_slice();
                this.bitxor_assign(that);
            }
            (this @ Encode::Run(_), that @ Encode::Bin(_)) => {
                this.as_boxed_slice();
                this.bitxor_assign(that);
            }
        }
    }
}

impl std::ops::Not for Encode {
    type Output = Encode;
    fn not(self) -> Self::Output {
        match self {
            Encode::Map(map) => Encode::Map(!map),
            Encode::Bin(bin) => Encode::Map(!bin),
            Encode::Run(run) => Encode::Run(!run),
        }
    }
}
impl std::ops::Not for &'_ Encode {
    type Output = Encode;
    fn not(self) -> Self::Output {
        match self {
            Encode::Map(ref map) => Encode::Map(!map),
            Encode::Bin(ref bin) => Encode::Map(!bin),
            Encode::Run(ref run) => Encode::Run(!run),
        }
    }
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
    fn partial_cmp(&self, rhs: &Pair) -> Option<std::cmp::Ordering> {
        self.value().partial_cmp(&rhs.value())
    }
}
impl Ord for Pair {
    fn cmp(&self, rhs: &Pair) -> std::cmp::Ordering {
        self.value().cmp(&rhs.value())
    }
}

// assume that each elements (range) has no overlap
fn merged<'a, 'b, 'r, L, R>(
    lhs: L,
    rhs: R,
) -> iter::MergeBy<
    impl Iterator<Item = Merged> + 'a,
    impl Iterator<Item = Merged> + 'b,
    impl Fn(&Merged, &Merged) -> std::cmp::Ordering,
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
    iter::MergeBy::merge_by(lhs, rhs, |a, b| match (a, b) {
        (Merged::Lhs(a), Merged::Lhs(b)) => a.cmp(b),
        (Merged::Lhs(a), Merged::Rhs(b)) => a.cmp(b),
        (Merged::Rhs(a), Merged::Lhs(b)) => a.cmp(b),
        (Merged::Rhs(a), Merged::Rhs(b)) => a.cmp(b),
    })
}

impl<'r> Members<'r> {
    fn new<'a, 'b>(
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

    fn filter_and(self) -> impl Iterator<Item = std::ops::Range<u32>> + 'r {
        self.filter_map(|member| match member {
            Member::And(range) => Some(range),
            _ => None,
        })
    }

    fn filter_or(self) -> impl Iterator<Item = std::ops::Range<u32>> + 'r {
        self.filter_map(|member| match member {
            Member::Lhs(range) => Some(range),
            Member::Rhs(range) => Some(range),
            Member::And(range) => Some(range),
            _ => None,
        })
    }

    fn filter_xor(self) -> impl Iterator<Item = std::ops::Range<u32>> + 'r {
        self.filter_map(|member| match member {
            Member::Lhs(range) => Some(range),
            Member::Rhs(range) => Some(range),
            _ => None,
        })
    }

    fn filter_not(self) -> impl Iterator<Item = std::ops::Range<u32>> + 'r {
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
            Box::new(iter::Tuples::tuples(merged))
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
                if range.end < BLOCK_SIZE as u32 {
                    Some(Member::Not(range.end..BLOCK_SIZE as u32))
                } else {
                    None
                }
            }

            // iterator is empty
            (None, None) => {
                self.finished = true;
                Some(Member::Not(0..BLOCK_SIZE as u32))
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

mod seq;
mod arr;
mod run;

mod impl_iter;
mod impl_dict;
mod impl_conv;
mod impl_bits;
mod impl_io;

use std::{cmp, fmt, mem, ops};
use std::iter::FromIterator;
use bits::{self, PopCount, Rank, Select0, Select1};
use self::Block::{Arr, Run, Seq};

pub const SEQ_MAX_LEN: usize = 4096;
pub const ARR_MAX_LEN: usize = 1024;

/// Internal representaions of a bits block.
#[derive(Clone, PartialEq, Eq)]
pub(crate) enum Block {
    Seq(SeqBlock),
    Arr(ArrBlock),
    Run(RunBlock),
}
#[derive(Clone, Default, PartialEq, Eq)]
pub(crate) struct SeqBlock {
    vector: Vec<u16>,
}
#[derive(Clone)]
pub(crate) struct ArrBlock {
    weight: u32,
    bitmap: Box<[u64; 1024]>,
}
#[derive(Clone, Default, PartialEq, Eq)]
pub(crate) struct RunBlock {
    weight: u32,
    ranges: Vec<Range>,
}

pub(crate) type Range = ops::RangeInclusive<u16>;

pub(crate) struct BoxedIter<'a> {
    rest: u32,
    iter: Box<Iterator<Item = u16> + 'a>,
}

pub(crate) struct OwnedIter {
    rest: u32,
    iter: Box<Iterator<Item = u16>>,
}

impl fmt::Debug for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Seq(_) => f.pad("Block::Seq"),
            Arr(_) => f.pad("Block::Arr"),
            Run(_) => f.pad("Block::Run"),
        }
    }
}
impl Default for Block {
    fn default() -> Self {
        Arr(ArrBlock::default())
    }
}

macro_rules! delegate {
    ( $this:ident, $method:ident $(, $args:expr )* ) => {
        {
            match $this {
                Seq(data) => data.$method( $( $args ),* ),
                Arr(data) => data.$method( $( $args ),* ),
                Run(data) => data.$method( $( $args ),* ),
            }
        }
    };
    ( ref $this:ident, $method:ident $(, $args:expr )* ) => {
        {
            match *$this {
                Seq(ref data) => data.$method( $( $args ),* ),
                Arr(ref data) => data.$method( $( $args ),* ),
                Run(ref data) => data.$method( $( $args ),* ),
            }
        }
    };
    ( ref mut $this: ident, $method: ident $(, $args: expr )* ) => {
        {
            match *$this {
                Seq(ref mut data) => data.$method( $( $args ),* ),
                Arr(ref mut data) => data.$method( $( $args ),* ),
                Run(ref mut data) => data.$method( $( $args ),* ),
            }
        }
    }
}

impl Block {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn clear(&mut self) {
        *self = Self::new();
    }
    pub fn contains(&self, bit: u16) -> bool {
        delegate!(ref self, contains, bit)
    }
    pub fn insert(&mut self, bit: u16) -> bool {
        delegate!(ref mut self, insert, bit)
    }
    pub fn remove(&mut self, bit: u16) -> bool {
        delegate!(ref mut self, remove, bit)
    }

    fn size_of_units() -> (usize, usize, usize, usize) {
        let size_of_u16 = mem::size_of::<u16>();
        let size_of_u32 = mem::size_of::<u32>();
        let size_of_u64 = mem::size_of::<u64>();
        let size_of_run = mem::size_of::<ops::RangeInclusive<u16>>();
        (size_of_u16, size_of_u32, size_of_u64, size_of_run)
    }

    /// Convert to more efficient inner representaions.
    pub fn optimize(&mut self) {
        let (size_of_u16, size_of_u32, size_of_u64, size_of_run) = Self::size_of_units();

        let new_repr = match *self {
            Seq(ref seq) => {
                let run = RunBlock::from(seq);
                let mem_in_seq = size_of_u16 * seq.vector.len();
                let mem_in_arr = size_of_u64 * ARR_MAX_LEN + size_of_u32;
                let mem_in_run = size_of_run * run.ranges.len() + size_of_u32;

                if mem_in_run <= cmp::min(mem_in_arr, mem_in_seq) {
                    Some(Run(run))
                } else if self.count1() as usize <= SEQ_MAX_LEN {
                    None
                } else {
                    Some(Arr(ArrBlock::from(seq)))
                }
            }

            Arr(ref arr) => {
                let run = RunBlock::from(arr);
                let mem_in_seq = size_of_u16 * (arr.weight as usize);
                let mem_in_arr = size_of_u64 * ARR_MAX_LEN + size_of_u32;
                let mem_in_run = size_of_run * run.ranges.len() + size_of_u32;

                if mem_in_run <= cmp::min(mem_in_arr, mem_in_seq) {
                    Some(Run(run))
                } else if arr.weight as usize <= SEQ_MAX_LEN {
                    Some(Seq(SeqBlock::from(arr)))
                } else {
                    None
                }
            }

            Run(ref run) => {
                let mem_in_seq = size_of_u16 * (run.weight as usize);
                let mem_in_arr = size_of_u64 * ARR_MAX_LEN + size_of_u32;
                let mem_in_run = size_of_run * run.ranges.len() + size_of_u32;

                if mem_in_run <= cmp::min(mem_in_arr, mem_in_seq) {
                    None
                } else if run.weight as usize <= SEQ_MAX_LEN {
                    Some(Seq(SeqBlock::from(run)))
                } else {
                    Some(Arr(ArrBlock::from(run)))
                }
            }
        };
        if let Some(repr) = new_repr {
            *self = repr;
        }
    }

    pub fn iter(&self) -> BoxedIter {
        self.into_iter()
    }
}

impl From<SeqBlock> for Block {
    fn from(seq: SeqBlock) -> Self {
        Seq(seq)
    }
}
impl From<ArrBlock> for Block {
    fn from(arr: ArrBlock) -> Self {
        Arr(arr)
    }
}
impl From<RunBlock> for Block {
    fn from(run: RunBlock) -> Self {
        Run(run)
    }
}

impl FromIterator<u16> for Block {
    fn from_iter<I>(iterable: I) -> Self
    where
        I: ::std::iter::IntoIterator<Item = u16>,
    {
        let iter = iterable.into_iter();
        let mut repr = Block::new();
        let ones = {
            let mut weight = 0;
            for item in iter {
                if !repr.insert(item) {
                    weight += 1;
                }
            }
            weight
        };
        debug_assert_eq!(ones, repr.count1());
        repr
    }
}

impl<'a> FromIterator<&'a u16> for Block {
    fn from_iter<I>(iterable: I) -> Self
    where
        I: ::std::iter::IntoIterator<Item = &'a u16>,
    {
        let iter = iterable.into_iter();
        iter.cloned().collect::<Self>()
    }
}

impl Block {
    fn as_arr(&mut self) {
        *self = match *self {
            Seq(ref seq) => Arr(ArrBlock::from(seq)),
            Run(ref run) => Arr(ArrBlock::from(run)),
            _ => unreachable!(),
        }
    }
}

impl<'a> IntoIterator for &'a Block {
    type Item = u16;
    type IntoIter = BoxedIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        delegate!(ref self, into_iter)
    }
}
impl IntoIterator for Block {
    type Item = u16;
    type IntoIter = OwnedIter;
    fn into_iter(self) -> Self::IntoIter {
        delegate!(self, into_iter)
    }
}

impl PopCount<u32> for Block {
    const SIZE: u32 = impl_dict::SIZE;
    fn count1(&self) -> u32 {
        delegate!(ref self, count1)
    }
}
impl Rank<u16> for Block {
    fn rank1(&self, i: u16) -> u16 {
        delegate!(ref self, rank1, i)
    }
}
impl Select1<u16> for Block {
    fn select1(&self, c: u16) -> Option<u16> {
        delegate!(ref self, select1, c)
    }
}
impl Select0<u16> for Block {
    fn select0(&self, c: u16) -> Option<u16> {
        delegate!(ref self, select0, c)
    }
}

impl<'a> bits::BitAndAssign<&'a Block> for Block {
    fn bitand_assign(&mut self, repr: &Block) {
        match *repr {
            Seq(ref b) => self.bitand_assign(b),
            Arr(ref b) => self.bitand_assign(b),
            Run(ref b) => self.bitand_assign(b),
        }
    }
}

impl<'a> bits::BitOrAssign<&'a Block> for Block {
    fn bitor_assign(&mut self, repr: &Block) {
        match *repr {
            Seq(ref b) => self.bitor_assign(b),
            Arr(ref b) => self.bitor_assign(b),
            Run(ref b) => self.bitor_assign(b),
        }
    }
}

impl<'a> bits::BitAndNotAssign<&'a Block> for Block {
    fn bitandnot_assign(&mut self, repr: &Block) {
        match *repr {
            Seq(ref b) => self.bitandnot_assign(b),
            Arr(ref b) => self.bitandnot_assign(b),
            Run(ref b) => self.bitandnot_assign(b),
        }
    }
}

impl<'a> bits::BitXorAssign<&'a Block> for Block {
    fn bitxor_assign(&mut self, repr: &Block) {
        match *repr {
            Seq(ref b) => self.bitxor_assign(b),
            Arr(ref b) => self.bitxor_assign(b),
            Run(ref b) => self.bitxor_assign(b),
        }
    }
}

impl<'a> bits::BitAndAssign<&'a SeqBlock> for Block {
    fn bitand_assign(&mut self, target: &SeqBlock) {
        match self {
            &mut Seq(ref mut b) => b.bitand_assign(target),
            &mut Arr(ref mut b) => b.bitand_assign(&ArrBlock::from(target)),
            this @ &mut Run(_) => {
                this.as_arr();
                this.bitand_assign(target);
            }
        }
    }
}

impl<'a> bits::BitOrAssign<&'a SeqBlock> for Block {
    fn bitor_assign(&mut self, target: &SeqBlock) {
        match self {
            &mut Seq(ref mut b) => b.bitor_assign(target),
            &mut Arr(ref mut b) => for &bit in &target.vector {
                b.insert(bit);
            },
            this @ &mut Run(_) => {
                this.as_arr();
                this.bitor_assign(target);
            }
        }
    }
}

impl<'a> bits::BitAndNotAssign<&'a SeqBlock> for Block {
    fn bitandnot_assign(&mut self, target: &SeqBlock) {
        match self {
            &mut Seq(ref mut b) => b.bitandnot_assign(target),
            &mut Arr(ref mut b) => for &bit in &target.vector {
                b.remove(bit);
            },
            this @ &mut Run(_) => {
                this.as_arr();
                this.bitandnot_assign(target);
            }
        }
    }
}

impl<'a> bits::BitXorAssign<&'a SeqBlock> for Block {
    fn bitxor_assign(&mut self, target: &SeqBlock) {
        match self {
            &mut Seq(ref mut b) => b.bitxor_assign(target),
            &mut Arr(ref mut b) => for &bit in &target.vector {
                if b.contains(bit) {
                    b.remove(bit);
                } else {
                    b.insert(bit);
                }
            },
            this @ &mut Run(_) => {
                this.as_arr();
                this.bitxor_assign(target);
            }
        }
    }
}

impl<'a> bits::BitAndAssign<&'a ArrBlock> for Block {
    fn bitand_assign(&mut self, target: &'a ArrBlock) {
        match self {
            &mut Seq(ref mut b) => {
                let mut n = 0;
                for i in 0..b.vector.len() {
                    if target.contains(b.vector[i]) {
                        b.vector[n] = b.vector[i];
                        n += 1;
                    }
                }
                b.vector.truncate(n);
            }

            &mut Arr(ref mut b) => b.bitand_assign(target),

            this @ &mut Run(_) => {
                this.as_arr();
                this.bitand_assign(target);
            }
        }
    }
}

impl<'a> bits::BitOrAssign<&'a ArrBlock> for Block {
    fn bitor_assign(&mut self, target: &ArrBlock) {
        match self {
            &mut Arr(ref mut b) => b.bitor_assign(target),
            this => {
                this.as_arr();
                this.bitor_assign(target);
            }
        }
    }
}

impl<'a> bits::BitAndNotAssign<&'a ArrBlock> for Block {
    fn bitandnot_assign(&mut self, target: &ArrBlock) {
        match self {
            &mut Arr(ref mut b) => b.bitandnot_assign(target),
            this => {
                this.as_arr();
                this.bitandnot_assign(target);
            }
        }
    }
}

impl<'a> bits::BitXorAssign<&'a ArrBlock> for Block {
    fn bitxor_assign(&mut self, target: &ArrBlock) {
        match self {
            &mut Arr(ref mut b) => b.bitxor_assign(target),
            this => {
                this.as_arr();
                this.bitxor_assign(target);
            }
        }
    }
}

impl<'a> bits::BitAndAssign<&'a RunBlock> for Block {
    fn bitand_assign(&mut self, target: &RunBlock) {
        match self {
            this @ &mut Seq(_) => {
                this.as_arr();
                this.bitand_assign(target);
            }
            &mut Arr(ref mut b) => b.bitand_assign(&ArrBlock::from(target)),
            &mut Run(ref mut b) => b.bitand_assign(target),
        }
    }
}

impl<'a> bits::BitOrAssign<&'a RunBlock> for Block {
    fn bitor_assign(&mut self, target: &RunBlock) {
        match self {
            this @ &mut Seq(_) => {
                this.as_arr();
                this.bitor_assign(target);
            }
            &mut Arr(ref mut b) => for range in &target.ranges {
                for bit in range.start..=range.end {
                    b.insert(bit);
                }
            },
            &mut Run(ref mut b) => b.bitor_assign(target),
        }
    }
}

impl<'a> bits::BitAndNotAssign<&'a RunBlock> for Block {
    fn bitandnot_assign(&mut self, target: &RunBlock) {
        match self {
            this @ &mut Seq(_) => {
                this.as_arr();
                this.bitandnot_assign(target);
            }
            &mut Arr(ref mut b) => for range in &target.ranges {
                for bit in range.start..=range.end {
                    b.remove(bit);
                }
            },
            &mut Run(ref mut b) => b.bitandnot_assign(target),
        }
    }
}

impl<'a> bits::BitXorAssign<&'a RunBlock> for Block {
    fn bitxor_assign(&mut self, target: &RunBlock) {
        match self {
            this @ &mut Seq(_) => {
                this.as_arr();
                this.bitxor_assign(target);
            }
            &mut Arr(ref mut b) => for range in &target.ranges {
                for bit in range.start..=range.end {
                    if b.contains(bit) {
                        b.remove(bit);
                    } else {
                        b.insert(bit);
                    }
                }
            },
            &mut Run(ref mut b) => b.bitxor_assign(target),
        }
    }
}

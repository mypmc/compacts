#[macro_use]
mod macros;
mod seq16;
mod arr64;
mod run16;
mod iter;

use std::{cmp, fmt, mem, ops};
use std::iter::FromIterator;
use bits::{self, PopCount, Rank, Select0, Select1};

use super::{Assign, Compare};
use self::Repr::{Arr, Run, Seq};

pub(crate) use self::iter::{Boxed as ReprBoxedIter, Owned as ReprOwnedIter};
pub(crate) use self::seq16::Seq16;
pub(crate) use self::arr64::Arr64;
pub(crate) use self::run16::Run16;

/// Stats of Block.
/// 'ones' is a count of non-zero bits.
/// 'size' is an approximate size in bytes.
// #[derive(Clone, Debug)]
// pub struct Stats {
//     pub ones: u64,
//     pub size: usize,
// }

/// Internal representaions of a bits block.
#[derive(Clone, PartialEq, Eq)]
pub(crate) enum Repr {
    Seq(Seq16),
    Arr(Arr64),
    Run(Run16),
}

impl Default for Repr {
    fn default() -> Self {
        Arr(Arr64::default())
    }
}

impl From<Seq16> for Repr {
    fn from(seq16: Seq16) -> Self {
        Seq(seq16)
    }
}
impl From<Arr64> for Repr {
    fn from(arr64: Arr64) -> Self {
        Arr(arr64)
    }
}
impl From<Run16> for Repr {
    fn from(run16: Run16) -> Self {
        Run(run16)
    }
}

impl FromIterator<u16> for Repr {
    fn from_iter<I>(iterable: I) -> Self
    where
        I: ::std::iter::IntoIterator<Item = u16>,
    {
        let iter = iterable.into_iter();
        let mut repr = Repr::new();
        let ones = {
            let mut weight = 0;
            for item in iter {
                if repr.insert(item) {
                    weight += 1;
                }
            }
            weight
        };
        debug_assert_eq!(ones, repr.count1());
        repr
    }
}

impl<'a> FromIterator<&'a u16> for Repr {
    fn from_iter<I>(iterable: I) -> Self
    where
        I: ::std::iter::IntoIterator<Item = &'a u16>,
    {
        let iter = iterable.into_iter();
        iter.cloned().collect::<Self>()
    }
}

impl fmt::Debug for Repr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Seq(ref b) => b.fmt(f),
            Arr(ref b) => b.fmt(f),
            Run(ref b) => b.fmt(f),
        }
    }
}

impl Repr {
    fn as_arr(&mut self) {
        *self = match *self {
            Seq(ref seq) => Arr(Arr64::from(seq)),
            Run(ref run) => Arr(Arr64::from(run)),
            _ => unreachable!(),
        }
    }
}

impl Repr {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }

    pub fn contains(&self, bit: u16) -> bool {
        match *self {
            Seq(ref seq) => seq.contains(bit),
            Arr(ref arr) => arr.contains(bit),
            Run(ref run) => run.contains(bit),
        }
    }

    pub fn insert(&mut self, bit: u16) -> bool {
        match *self {
            Seq(ref mut seq) => seq.insert(bit),
            Arr(ref mut arr) => arr.insert(bit),
            Run(ref mut run) => run.insert(bit),
        }
    }

    pub fn remove(&mut self, bit: u16) -> bool {
        match *self {
            Seq(ref mut seq) => seq.remove(bit),
            Arr(ref mut arr) => arr.remove(bit),
            Run(ref mut run) => run.remove(bit),
        }
    }

    pub fn shrink(&mut self) {
        match *self {
            Seq(ref mut seq) => seq.vector.shrink_to_fit(),
            Arr(ref mut _arr) => { /* ignore */ }
            Run(ref mut run) => run.ranges.shrink_to_fit(),
        }
    }

    fn size_of_units() -> (usize, usize, usize, usize) {
        let size_of_u16 = mem::size_of::<u16>();
        let size_of_u32 = mem::size_of::<u32>();
        let size_of_u64 = mem::size_of::<u64>();
        let size_of_run = mem::size_of::<ops::RangeInclusive<u16>>();
        (size_of_u16, size_of_u32, size_of_u64, size_of_run)
    }

    pub fn mem_size(&self) -> usize {
        let (size_of_u16, size_of_u32, size_of_u64, size_of_run) = Self::size_of_units();

        match *self {
            Seq(ref seq) => size_of_u16 * seq.vector.len(),
            Arr(ref arr) => size_of_u32 + size_of_u64 * arr.boxarr.len(),
            Run(ref run) => size_of_u32 + size_of_run * run.ranges.len(),
        }
    }

    /// Convert to more efficient inner representaions.
    pub fn optimize(&mut self) {
        let (size_of_u16, size_of_u32, size_of_u64, size_of_run) = Self::size_of_units();

        let new_repr = match *self {
            Seq(ref seq) => {
                let run = Run16::from(seq);
                let mem_in_seq16 = size_of_u16 * seq.vector.len();
                let mem_in_arr64 = size_of_u64 * bits::ARR_MAX_LEN + size_of_u32;
                let mem_in_run16 = size_of_run * run.ranges.len() + size_of_u32;

                if mem_in_run16 <= cmp::min(mem_in_arr64, mem_in_seq16) {
                    Some(Run(run))
                } else if self.count1() as usize <= bits::SEQ_MAX_LEN {
                    None
                } else {
                    Some(Arr(Arr64::from(seq)))
                }
            }

            Arr(ref arr) => {
                let run = Run16::from(arr);
                let mem_in_seq16 = size_of_u16 * (arr.weight as usize);
                let mem_in_arr64 = size_of_u64 * bits::ARR_MAX_LEN + size_of_u32;
                let mem_in_run16 = size_of_run * run.ranges.len() + size_of_u32;

                if mem_in_run16 <= cmp::min(mem_in_arr64, mem_in_seq16) {
                    Some(Run(run))
                } else if arr.weight as usize <= bits::SEQ_MAX_LEN {
                    Some(Seq(Seq16::from(arr)))
                } else {
                    None
                }
            }

            Run(ref run) => {
                let mem_in_seq16 = size_of_u16 * (run.weight as usize);
                let mem_in_arr64 = size_of_u64 * bits::ARR_MAX_LEN + size_of_u32;
                let mem_in_run16 = size_of_run * run.ranges.len() + size_of_u32;

                if mem_in_run16 <= cmp::min(mem_in_arr64, mem_in_seq16) {
                    None
                } else if run.weight as usize <= bits::SEQ_MAX_LEN {
                    Some(Seq(Seq16::from(run)))
                } else {
                    Some(Arr(Arr64::from(run)))
                }
            }
        };
        if let Some(repr) = new_repr {
            *self = repr;
        }
    }

    // pub fn stats(&self) -> Stats {
    //     let ones = u64::from(self.count1());
    //     let size = self.mem_size();
    //     Stats { ones, size }
    // }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = u16> + 'a {
        self.into_iter()
    }
}

impl<'a> IntoIterator for &'a Repr {
    type Item = u16;
    type IntoIter = ReprBoxedIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        match *self {
            Seq(ref seq) => seq.into_iter(),
            Arr(ref arr) => arr.into_iter(),
            Run(ref run) => run.into_iter(),
        }
    }
}
impl IntoIterator for Repr {
    type Item = u16;
    type IntoIter = ReprOwnedIter;
    fn into_iter(self) -> Self::IntoIter {
        match self {
            Seq(seq) => seq.into_iter(),
            Arr(arr) => arr.into_iter(),
            Run(run) => run.into_iter(),
        }
    }
}

impl PopCount<u32> for Repr {
    const SIZE: u32 = 1 << 16;

    fn count1(&self) -> u32 {
        match *self {
            Seq(ref seq) => seq.vector.len() as u32,
            Arr(ref arr) => arr.weight,
            Run(ref run) => run.weight,
        }
    }
}

impl Rank<u16> for Repr {
    fn rank1(&self, i: u16) -> u16 {
        match *self {
            Seq(ref seq) => {
                let vec = &seq.vector;
                let fun = |p| vec.get(p).map_or(false, |&v| v >= i);
                search!(0, vec.len(), fun) as u16
            }

            Arr(ref arr) => {
                let q = (i / 64) as usize;
                let r = u32::from(i % 64);
                let vec = &arr.boxarr;
                let init = vec.iter().take(q).fold(0, |acc, w| {
                    let c1: u16 = w.count1();
                    acc + c1
                });
                let last = vec.get(q).map_or(0, |w| w.rank1(r) as u16);
                init + last
            }

            Run(ref run) => match run.search(&i) {
                Err(n) => if n >= run.ranges.len() {
                    run.weight as u16
                } else {
                    run.ranges
                        .iter()
                        .map(|r| r.end - r.start + 1)
                        .take(n)
                        .sum::<u16>()
                },
                Ok(n) => {
                    let r = run.ranges
                        .iter()
                        .map(|r| r.end - r.start + 1)
                        .take(n)
                        .sum::<u16>();
                    i - run.ranges[n].start + r
                }
            },
        }
    }
}

impl Select1<u16> for Repr {
    fn select1(&self, c: u16) -> Option<u16> {
        if u32::from(c) >= self.count1() {
            return None;
        }
        match *self {
            Seq(ref seq) => seq.vector.get(c as usize).cloned(),

            Arr(ref arr) => {
                let mut remain = u32::from(c);
                for (i, bit) in arr.boxarr.iter().enumerate().filter(|&(_, v)| *v != 0) {
                    let ones = bit.count1();
                    if remain < ones {
                        let width = 64;
                        let select = bit.select1(remain).unwrap_or(0);
                        return Some((width * i) as u16 + select as u16);
                    }
                    remain -= ones;
                }
                None
            }

            Run(ref run) => {
                let mut curr = 0;
                for range in &run.ranges {
                    let next = curr + (range.end - range.start + 1);
                    if next > c {
                        return Some(range.start - curr + c);
                    }
                    curr = next;
                }
                None
            }
        }
    }
}

impl Select0<u16> for Repr {
    fn select0(&self, c: u16) -> Option<u16> {
        if u32::from(c) >= self.count0() {
            return None;
        }
        match *self {
            Seq(_) | Run(_) => select_by_rank!(0, self, c, 0u32, 1 << 16, u16),

            Arr(ref arr) => {
                let mut remain = u32::from(c);
                for (i, bit) in arr.boxarr.iter().enumerate() {
                    let zeros = bit.count0();
                    if remain < zeros {
                        let width = 64;
                        let select = bit.select0(remain).unwrap_or(0);
                        return Some((width * i) as u16 + select as u16);
                    }
                    remain -= zeros;
                }
                None
            }
        }
    }
}

impl<'a> Assign<&'a Repr> for Repr {
    fn and_assign(&mut self, repr: &Repr) {
        match *repr {
            Seq(ref b) => self.and_assign(b),
            Arr(ref b) => self.and_assign(b),
            Run(ref b) => self.and_assign(b),
        }
    }

    fn or_assign(&mut self, repr: &Repr) {
        match *repr {
            Seq(ref b) => self.or_assign(b),
            Arr(ref b) => self.or_assign(b),
            Run(ref b) => self.or_assign(b),
        }
    }

    fn and_not_assign(&mut self, repr: &Repr) {
        match *repr {
            Seq(ref b) => self.and_not_assign(b),
            Arr(ref b) => self.and_not_assign(b),
            Run(ref b) => self.and_not_assign(b),
        }
    }

    fn xor_assign(&mut self, repr: &Repr) {
        match *repr {
            Seq(ref b) => self.xor_assign(b),
            Arr(ref b) => self.xor_assign(b),
            Run(ref b) => self.xor_assign(b),
        }
    }
}

impl<'a> Assign<&'a Seq16> for Repr {
    fn and_assign(&mut self, target: &Seq16) {
        match self {
            &mut Seq(ref mut b) => b.and_assign(target),
            &mut Arr(ref mut b) => b.and_assign(&Arr64::from(target)),
            this @ &mut Run(_) => {
                this.as_arr();
                this.and_assign(target);
            }
        }
    }

    fn or_assign(&mut self, target: &Seq16) {
        match self {
            &mut Seq(ref mut b) => b.or_assign(target),
            &mut Arr(ref mut b) => for &bit in &target.vector {
                b.insert(bit);
            },
            this @ &mut Run(_) => {
                this.as_arr();
                this.or_assign(target);
            }
        }
    }

    fn and_not_assign(&mut self, target: &Seq16) {
        match self {
            &mut Seq(ref mut b) => b.and_not_assign(target),
            &mut Arr(ref mut b) => for &bit in &target.vector {
                b.remove(bit);
            },
            this @ &mut Run(_) => {
                this.as_arr();
                this.and_not_assign(target);
            }
        }
    }

    fn xor_assign(&mut self, target: &Seq16) {
        match self {
            &mut Seq(ref mut b) => b.xor_assign(target),
            &mut Arr(ref mut b) => for &bit in &target.vector {
                if b.contains(bit) {
                    b.remove(bit);
                } else {
                    b.insert(bit);
                }
            },
            this @ &mut Run(_) => {
                this.as_arr();
                this.xor_assign(target);
            }
        }
    }
}

impl<'a> Assign<&'a Arr64> for Repr {
    fn and_assign(&mut self, target: &'a Arr64) {
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

            &mut Arr(ref mut b) => b.and_assign(target),

            this @ &mut Run(_) => {
                this.as_arr();
                this.and_assign(target);
            }
        }
    }

    fn or_assign(&mut self, target: &Arr64) {
        match self {
            &mut Arr(ref mut b) => b.or_assign(target),
            this => {
                this.as_arr();
                this.or_assign(target);
            }
        }
    }

    fn and_not_assign(&mut self, target: &Arr64) {
        match self {
            &mut Arr(ref mut b) => b.and_not_assign(target),
            this => {
                this.as_arr();
                this.and_not_assign(target);
            }
        }
    }

    fn xor_assign(&mut self, target: &Arr64) {
        match self {
            &mut Arr(ref mut b) => b.xor_assign(target),
            this => {
                this.as_arr();
                this.xor_assign(target);
            }
        }
    }
}

impl<'a> Assign<&'a Run16> for Repr {
    fn and_assign(&mut self, target: &Run16) {
        match self {
            this @ &mut Seq(_) => {
                this.as_arr();
                this.and_assign(target);
            }
            &mut Arr(ref mut b) => b.and_assign(&Arr64::from(target)),
            &mut Run(ref mut b) => b.and_assign(target),
        }
    }

    fn or_assign(&mut self, target: &Run16) {
        match self {
            this @ &mut Seq(_) => {
                this.as_arr();
                this.or_assign(target);
            }
            &mut Arr(ref mut b) => for range in &target.ranges {
                for bit in range.start..=range.end {
                    b.insert(bit);
                }
            },
            &mut Run(ref mut b) => b.or_assign(target),
        }
    }

    fn and_not_assign(&mut self, target: &Run16) {
        match self {
            this @ &mut Seq(_) => {
                this.as_arr();
                this.and_not_assign(target);
            }
            &mut Arr(ref mut b) => for range in &target.ranges {
                for bit in range.start..=range.end {
                    b.remove(bit);
                }
            },
            &mut Run(ref mut b) => b.and_not_assign(target),
        }
    }

    fn xor_assign(&mut self, target: &Run16) {
        match self {
            this @ &mut Seq(_) => {
                this.as_arr();
                this.xor_assign(target);
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
            &mut Run(ref mut b) => b.xor_assign(target),
        }
    }
}

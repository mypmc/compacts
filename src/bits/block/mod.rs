mod seq16;
mod arr64;
mod run16;
mod iter;

use std::iter::{FromIterator, IntoIterator};
use std::{cmp, fmt, mem, ops};
use bits::dict::{PopCount, Rank, Select0, Select1};
use bits::pair::Assign;

pub(crate) use self::seq16::Seq16;
pub(crate) use self::arr64::Arr64;
pub(crate) use self::run16::Run16;

/// Internal representaions of a bits block.
#[derive(Clone, PartialEq, Eq)]
pub(crate) enum Block {
    Seq16(Seq16),
    Arr64(Arr64),
    Run16(Run16),
}
impl fmt::Debug for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Block::Seq16(_) => write!(f, "Seq16({})", self.count1()),
            Block::Arr64(_) => write!(f, "Arr64({})", self.count1()),
            Block::Run16(_) => write!(f, "Run16({})", self.count1()),
        }
    }
}

/// Stats of Block.
/// 'ones' is a count of non-zero bits.
/// 'size' is an approximate size in bytes.
#[derive(Clone, Debug)]
pub struct Stats {
    pub ones: u64,
    pub size: usize,
}

impl Default for Block {
    fn default() -> Self {
        Block::Arr64(Arr64::default())
    }
}

impl Block {
    const CAPACITY: usize = 1 << 16;

    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }

    pub fn contains(&self, bit: u16) -> bool {
        match *self {
            Block::Seq16(ref seq) => seq.contains(bit),
            Block::Arr64(ref arr) => arr.contains(bit),
            Block::Run16(ref run) => run.contains(bit),
        }
    }

    pub fn insert(&mut self, bit: u16) -> bool {
        match *self {
            Block::Seq16(ref mut seq) => seq.insert(bit),
            Block::Arr64(ref mut arr) => arr.insert(bit),
            Block::Run16(ref mut run) => run.insert(bit),
        }
    }

    pub fn remove(&mut self, bit: u16) -> bool {
        match *self {
            Block::Seq16(ref mut seq) => seq.remove(bit),
            Block::Arr64(ref mut arr) => arr.remove(bit),
            Block::Run16(ref mut run) => run.remove(bit),
        }
    }

    pub fn shrink(&mut self) {
        match *self {
            Block::Seq16(ref mut seq) => seq.vector.shrink_to_fit(),
            Block::Arr64(ref mut _arr) => { /* ignore */ }
            Block::Run16(ref mut run) => run.ranges.shrink_to_fit(),
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
            Block::Seq16(ref seq) => size_of_u16 * seq.vector.len(),
            Block::Arr64(ref arr) => size_of_u32 + size_of_u64 * arr.vector.len(),
            Block::Run16(ref run) => size_of_u32 + size_of_run * run.ranges.len(),
        }
    }

    fn as_arr64(&mut self) {
        *self = match *self {
            Block::Seq16(ref seq) => Block::Arr64(Arr64::from(seq)),
            Block::Run16(ref run) => Block::Arr64(Arr64::from(run)),
            _ => unreachable!(),
        }
    }

    /// Convert to more efficient block representaions.
    pub fn optimize(&mut self) {
        const SEQ16: usize = 4096; // 4096 * 16 == 65536
        const SEQ64: usize = 1024; // 1024 * 64 == 65536

        let (size_of_u16, size_of_u32, size_of_u64, size_of_run) = Self::size_of_units();

        let new_block = match *self {
            Block::Seq16(ref seq) => {
                let run = Run16::from(seq);
                let mem_in_seq16 = size_of_u16 * seq.vector.len();
                let mem_in_arr64 = size_of_u64 * SEQ64 + size_of_u32;
                let mem_in_run16 = size_of_run * run.ranges.len() + size_of_u32;

                if mem_in_run16 <= cmp::min(mem_in_arr64, mem_in_seq16) {
                    Some(Block::Run16(run))
                } else if self.count1() as usize <= SEQ16 {
                    None
                } else {
                    Some(Block::Arr64(Arr64::from(seq)))
                }
            }

            Block::Arr64(ref arr) => {
                let run = Run16::from(arr);
                let mem_in_seq16 = size_of_u16 * (arr.weight as usize);
                let mem_in_arr64 = size_of_u64 * SEQ64 + size_of_u32;
                let mem_in_run16 = size_of_run * run.ranges.len() + size_of_u32;

                if mem_in_run16 <= cmp::min(mem_in_arr64, mem_in_seq16) {
                    Some(Block::Run16(run))
                } else if arr.weight as usize <= SEQ16 {
                    Some(Block::Seq16(Seq16::from(arr)))
                } else {
                    None
                }
            }

            Block::Run16(ref run) => {
                let mem_in_seq16 = size_of_u16 * (run.weight as usize);
                let mem_in_arr64 = size_of_u64 * SEQ64 + size_of_u32;
                let mem_in_run16 = size_of_run * run.ranges.len() + size_of_u32;

                if mem_in_run16 <= cmp::min(mem_in_arr64, mem_in_seq16) {
                    None
                } else if run.weight as usize <= SEQ16 {
                    Some(Block::Seq16(Seq16::from(run)))
                } else {
                    Some(Block::Arr64(Arr64::from(run)))
                }
            }
        };
        if let Some(block) = new_block {
            *self = block;
        }
    }

    pub fn stats(&self) -> Stats {
        let ones = u64::from(self.count1());
        let size = self.mem_size();
        Stats { ones, size }
    }

    pub fn iter(&self) -> iter::Boxed {
        self.into_iter()
    }
}

impl<'a> IntoIterator for &'a Block {
    type Item = u16;
    type IntoIter = iter::Boxed<'a>;
    fn into_iter(self) -> Self::IntoIter {
        match *self {
            Block::Seq16(ref seq) => seq.into_iter(),
            Block::Arr64(ref arr) => arr.into_iter(),
            Block::Run16(ref run) => run.into_iter(),
        }
    }
}
impl IntoIterator for Block {
    type Item = u16;
    type IntoIter = iter::Owned;
    fn into_iter(self) -> Self::IntoIter {
        match self {
            Block::Seq16(seq) => seq.into_iter(),
            Block::Arr64(arr) => arr.into_iter(),
            Block::Run16(run) => run.into_iter(),
        }
    }
}


impl PopCount<u32> for Block {
    const SIZE: u32 = 1 << 16;

    fn count1(&self) -> u32 {
        match *self {
            Block::Seq16(ref seq) => seq.vector.len() as u32,
            Block::Arr64(ref arr) => arr.weight,
            Block::Run16(ref run) => run.weight,
        }
    }
}

impl Rank<u16> for Block {
    fn rank1(&self, i: u16) -> u16 {
        match *self {
            Block::Seq16(ref seq) => {
                let vec = &seq.vector;
                let fun = |p| vec.get(p).map_or(false, |&v| v >= i);
                search!(0, vec.len(), fun) as u16
            }

            Block::Arr64(ref arr) => {
                let q = (i / 64) as usize;
                let r = u32::from(i % 64);
                let vec = &arr.vector;
                let init = vec.iter().take(q).fold(0, |acc, w| {
                    let c1: u16 = w.count1();
                    acc + c1
                });
                let last = vec.get(q).map_or(0, |w| w.rank1(r) as u16);
                init + last
            }

            Block::Run16(ref run) => match run.search(&i) {
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

impl Select1<u16> for Block {
    fn select1(&self, c: u16) -> Option<u16> {
        if u32::from(c) >= self.count1() {
            return None;
        }
        match *self {
            Block::Seq16(ref seq) => seq.vector.get(c as usize).cloned(),

            Block::Arr64(ref arr) => {
                let mut remain = u32::from(c);
                for (i, bit) in arr.vector.iter().enumerate().filter(|&(_, v)| *v != 0) {
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

            Block::Run16(ref run) => {
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

impl Select0<u16> for Block {
    fn select0(&self, c: u16) -> Option<u16> {
        if u32::from(c) >= self.count0() {
            return None;
        }
        match *self {
            Block::Seq16(_) | Block::Run16(_) => select_by_rank!(0, self, c, 0u32, 1 << 16, u16),

            Block::Arr64(ref arr) => {
                let mut remain = u32::from(c);
                for (i, bit) in arr.vector.iter().enumerate() {
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

impl<'a> Assign<&'a Block> for Block {
    fn and_assign(&mut self, block: &Block) {
        match *block {
            Block::Seq16(ref seq) => self.and_assign(seq),
            Block::Arr64(ref arr) => self.and_assign(arr),
            Block::Run16(ref run) => self.and_assign(run),
        }
    }

    fn or_assign(&mut self, block: &Block) {
        match *block {
            Block::Seq16(ref seq) => self.or_assign(seq),
            Block::Arr64(ref arr) => self.or_assign(arr),
            Block::Run16(ref run) => self.or_assign(run),
        }
    }

    fn and_not_assign(&mut self, block: &Block) {
        match *block {
            Block::Seq16(ref seq) => self.and_not_assign(seq),
            Block::Arr64(ref arr) => self.and_not_assign(arr),
            Block::Run16(ref run) => self.and_not_assign(run),
        }
    }

    fn xor_assign(&mut self, block: &Block) {
        match *block {
            Block::Seq16(ref seq) => self.xor_assign(seq),
            Block::Arr64(ref arr) => self.xor_assign(arr),
            Block::Run16(ref run) => self.xor_assign(run),
        }
    }
}

impl<'a> Assign<&'a Seq16> for Block {
    fn and_assign(&mut self, target: &'a Seq16) {
        match self {
            &mut Block::Seq16(ref mut b) => b.and_assign(target),
            &mut Block::Arr64(ref mut b) => b.and_assign(&Arr64::from(target)),
            this @ &mut Block::Run16(_) => {
                this.as_arr64();
                this.and_assign(target);
            }
        }
    }

    fn or_assign(&mut self, target: &Seq16) {
        match self {
            &mut Block::Seq16(ref mut b) => b.or_assign(target),
            &mut Block::Arr64(ref mut b) => for &bit in &target.vector {
                b.insert(bit);
            },
            this @ &mut Block::Run16(_) => {
                this.as_arr64();
                this.or_assign(target);
            }
        }
    }

    fn and_not_assign(&mut self, target: &Seq16) {
        match self {
            &mut Block::Seq16(ref mut b) => b.and_not_assign(target),
            &mut Block::Arr64(ref mut b) => for &bit in &target.vector {
                b.remove(bit);
            },
            this @ &mut Block::Run16(_) => {
                this.as_arr64();
                this.and_not_assign(target);
            }
        }
    }

    fn xor_assign(&mut self, target: &Seq16) {
        match self {
            &mut Block::Seq16(ref mut b) => b.xor_assign(target),
            &mut Block::Arr64(ref mut b) => for &bit in &target.vector {
                if b.contains(bit) {
                    b.remove(bit);
                } else {
                    b.insert(bit);
                }
            },
            this @ &mut Block::Run16(_) => {
                this.as_arr64();
                this.xor_assign(target);
            }
        }
    }
}

impl<'a> Assign<&'a Arr64> for Block {
    fn and_assign(&mut self, target: &'a Arr64) {
        match self {
            &mut Block::Seq16(ref mut b) => {
                let mut n = 0;
                for i in 0..b.vector.len() {
                    if target.contains(b.vector[i]) {
                        b.vector[n] = b.vector[i];
                        n += 1;
                    }
                }
                b.vector.truncate(n);
            }

            &mut Block::Arr64(ref mut b) => b.and_assign(target),

            this @ &mut Block::Run16(_) => {
                this.as_arr64();
                this.and_assign(target);
            }
        }
    }

    fn or_assign(&mut self, target: &Arr64) {
        match self {
            &mut Block::Arr64(ref mut b) => b.or_assign(target),
            this => {
                this.as_arr64();
                this.or_assign(target);
            }
        }
    }

    fn and_not_assign(&mut self, target: &Arr64) {
        match self {
            &mut Block::Arr64(ref mut b) => b.and_not_assign(target),
            this => {
                this.as_arr64();
                this.and_not_assign(target);
            }
        }
    }

    fn xor_assign(&mut self, target: &Arr64) {
        match self {
            &mut Block::Arr64(ref mut b) => b.xor_assign(target),
            this => {
                this.as_arr64();
                this.xor_assign(target);
            }
        }
    }
}

impl<'a> Assign<&'a Run16> for Block {
    fn and_assign(&mut self, target: &'a Run16) {
        match self {
            this @ &mut Block::Seq16(_) => {
                this.as_arr64();
                this.and_assign(target);
            }
            &mut Block::Arr64(ref mut b) => b.and_assign(&Arr64::from(target)),
            &mut Block::Run16(ref mut b) => b.and_assign(target),
        }
    }

    fn or_assign(&mut self, target: &Run16) {
        match self {
            this @ &mut Block::Seq16(_) => {
                this.as_arr64();
                this.or_assign(target);
            }
            &mut Block::Arr64(ref mut b) => for range in &target.ranges {
                for bit in range.start..=range.end {
                    b.insert(bit);
                }
            },
            &mut Block::Run16(ref mut b) => b.or_assign(target),
        }
    }

    fn and_not_assign(&mut self, target: &Run16) {
        match self {
            this @ &mut Block::Seq16(_) => {
                this.as_arr64();
                this.and_not_assign(target);
            }
            &mut Block::Arr64(ref mut b) => for range in &target.ranges {
                for bit in range.start..=range.end {
                    b.remove(bit);
                }
            },
            &mut Block::Run16(ref mut b) => b.and_not_assign(target),
        }
    }

    fn xor_assign(&mut self, target: &Run16) {
        match self {
            this @ &mut Block::Seq16(_) => {
                this.as_arr64();
                this.xor_assign(target);
            }
            &mut Block::Arr64(ref mut b) => for range in &target.ranges {
                for bit in range.start..=range.end {
                    if b.contains(bit) {
                        b.remove(bit);
                    } else {
                        b.insert(bit);
                    }
                }
            },
            &mut Block::Run16(ref mut b) => b.xor_assign(target),
        }
    }
}

impl FromIterator<u16> for Block {
    fn from_iter<I>(iterable: I) -> Self
    where
        I: ::std::iter::IntoIterator<Item = u16>,
    {
        let iter = iterable.into_iter();
        let mut block = Block::new();
        let ones = extend_by_u16!(&mut block, iter);
        debug_assert_eq!(ones, block.count1());
        block
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

#[macro_use]
mod macros;
mod prim;
mod bitops;
mod pair;
mod entry;
mod repr;
mod dict;
#[cfg(test)]
mod tests;

use std::{fmt, io, iter, mem, ops};
use io::{read_from, ReadFrom, WriteTo};

use self::prim::{Merge, Split};
use self::repr::{Arr64, Repr, Run16, Seq16};
use self::bitops::{BitAndAssign, BitAndNotAssign, BitOrAssign, BitXorAssign};
pub use self::dict::{PopCount, Rank, Select0, Select1};
pub use self::entry::{Entries, Entry};
pub use self::entry::{And, AndNot, Or, Xor};
pub use self::entry::{and, and_not, or, xor};

/// Set of u32.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Set {
    blocks: Vec<Block>,
}
#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct Block {
    slot: u16,
    repr: Repr,
}

const SEQ_MAX_LEN: usize = 4096;
const ARR_MAX_LEN: usize = 1024;
const U64_BITSIZE: usize = 64;

impl Set {
    const TRUE: bool = true;

    const FALSE: bool = false;

    /// Return new Set.
    pub fn new() -> Self {
        Set { blocks: Vec::new() }
    }

    /// Set flag at `x`, and return a **previous** value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// #[macro_use]
    /// extern crate compacts;
    /// fn main() {
    ///     let mut bits = bits![1, 2, 8];
    ///     assert!(!bits.set(0, false));
    ///     assert!(bits.set(1, false));
    ///     assert!(!bits.set(1, true));
    ///     assert!(bits.set(1, true));
    /// }
    /// ```
    pub fn set(&mut self, x: u32, flag: bool) -> bool {
        if flag {
            !self.insert(x)
        } else {
            self.remove(x)
        }
    }

    fn search(&self, key: u16) -> Result<usize, usize> {
        self.blocks.binary_search_by_key(&key, |block| block.slot)
    }

    /// Clear contents from set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// #[macro_use]
    /// extern crate compacts;
    /// fn main() {
    ///     use compacts::bits::PopCount;
    ///     let mut bits = bits!(0);
    ///     assert!(bits.count1() == 1);
    ///     bits.clear();
    ///     assert!(bits.count1() == 0);
    /// }
    /// ```
    pub fn clear(&mut self) {
        self.blocks.clear();
    }

    /// Return `true` if `x` exists.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compacts::bits::{Set, PopCount};
    ///
    /// let mut bits = Set::new();
    /// assert_eq!(bits.count0(), 1 << 32);
    /// bits.insert(1);
    /// assert!(!bits.contains(0));
    /// assert!(bits.contains(1));
    /// assert!(!bits.contains(2));
    /// assert_eq!(bits.count1(), 1);
    /// ```
    pub fn contains(&self, x: u32) -> bool {
        let (slot, bit) = x.split();
        self.search(slot)
            .map(|i| self.blocks[i].repr.contains(bit))
            .unwrap_or(false)
    }

    /// Equivalent to `!set(x, true)`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compacts::bits::{Set, PopCount};
    ///
    /// let mut bits = Set::new();
    /// assert!(bits.insert(3));
    /// assert!(!bits.insert(3));
    /// assert!(bits.contains(3));
    /// assert_eq!(bits.count1(), 1);
    /// ```
    pub fn insert(&mut self, x: u32) -> bool {
        let (slot, bit) = x.split();
        let pos = self.search(slot);
        match pos {
            Ok(i) => {
                let block = &mut self.blocks[i];
                block.repr.insert(bit)
            }
            Err(i) => {
                let mut repr = Repr::new();
                repr.insert(bit);
                self.blocks.insert(i, Block { slot, repr });
                true
            }
        }
    }

    /// Equivalent to `set(x, false)`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compacts::bits::{Set, PopCount};
    ///
    /// let mut bits = Set::new();
    /// assert!(bits.insert(3));
    /// assert!(bits.remove(3));
    /// assert!(!bits.contains(3));
    /// assert_eq!(bits.count1(), 0);
    /// ```
    pub fn remove(&mut self, x: u32) -> bool {
        let (slot, bit) = x.split();
        let pos = self.search(slot);
        match pos {
            Ok(i) => {
                let block = &mut self.blocks[i];
                block.repr.remove(bit)
            }
            Err(_) => false,
        }
    }

    /// Optimize an innternal representaions.
    pub fn optimize(&mut self) {
        for block in &mut self.blocks {
            block.repr.optimize();
        }
        self.blocks.retain(|block| block.repr.count1() > 0);
        self.blocks.shrink_to_fit();
    }

    pub fn entries<'a>(&'a self) -> Entries<'a> {
        self.into_iter()
    }

    pub fn bits<'a>(&'a self) -> impl Iterator<Item = u32> + 'a {
        self.blocks.iter().flat_map(|block| {
            let slot = block.slot;
            block
                .repr
                .iter()
                .map(move |half| <u32 as Merge>::merge((slot, half)))
        })
    }

    pub fn and<'a, T>(&'a self, that: T) -> And<'a, Entries<'a>, T::IntoIter>
    where
        T: IntoIterator<Item = Entry<'a>>,
    {
        and(self, that)
    }
    pub fn or<'a, T>(&'a self, that: T) -> Or<'a, Entries<'a>, T::IntoIter>
    where
        T: IntoIterator<Item = Entry<'a>>,
    {
        or(self, that)
    }
    pub fn and_not<'a, T>(&'a self, that: T) -> AndNot<'a, Entries<'a>, T::IntoIter>
    where
        T: IntoIterator<Item = Entry<'a>>,
    {
        and_not(self, that)
    }
    pub fn xor<'a, T>(&'a self, that: T) -> Xor<'a, Entries<'a>, T::IntoIter>
    where
        T: IntoIterator<Item = Entry<'a>>,
    {
        xor(self, that)
    }
}

impl ops::Index<u32> for Set {
    type Output = bool;

    /// # Examples
    ///
    /// ```rust
    /// #[macro_use]
    /// extern crate compacts;
    /// fn main() {
    ///     let bits = bits!(0, 1 << 30);
    ///     assert!(bits[0]);
    ///     assert!(!bits[1 << 10]);
    ///     assert!(!bits[1 << 20]);
    ///     assert!(bits[1 << 30]);
    /// }
    /// ```
    fn index(&self, i: u32) -> &Self::Output {
        if self.contains(i) {
            &Self::TRUE
        } else {
            &Self::FALSE
        }
    }
}

impl<T: AsRef<[u32]>> From<T> for Set {
    fn from(v: T) -> Self {
        v.as_ref().iter().collect()
    }
}

impl<'a> iter::FromIterator<u32> for Set {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = u32>,
    {
        let mut bs = Set::new();
        for b in iter {
            bs.insert(b);
        }
        bs.optimize();
        bs
    }
}

impl<'a> iter::FromIterator<&'a u32> for Set {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a u32>,
    {
        let mut bs = Set::new();
        for b in iter {
            bs.insert(*b);
        }
        bs.optimize();
        bs
    }
}

impl PopCount<u64> for Set {
    const SIZE: u64 = 1 << 32;

    /// # Examples
    ///
    /// ```rust
    /// #[macro_use]
    /// extern crate compacts;
    /// fn main() {
    ///     use compacts::bits::PopCount;
    ///     let bits = bits![0, 1, 4, 1 << 8, 1 << 16];
    ///     assert_eq!(bits.count1(), 5);
    /// }
    /// ```
    fn count1(&self) -> u64 {
        self.blocks.iter().map(|b| u64::from(b.repr.count1())).sum()
    }
}

impl Rank<u32> for Set {
    /// # Examples
    ///
    /// ```rust
    /// #[macro_use]
    /// extern crate compacts;
    /// fn main() {
    ///     use compacts::bits::Rank;
    ///     let bits = bits![0, 1, 4, 1 << 8, 1 << 16];
    ///     assert_eq!(bits.rank1(0), 0);
    ///     assert_eq!(bits.rank1(1), 1);
    ///     assert_eq!(bits.rank1(2), 2);
    ///     assert_eq!(bits.rank1(3), 2);
    ///     assert_eq!(bits.rank1(4), 2);
    ///     assert_eq!(bits.rank1(5), 3);
    /// }
    /// ```
    fn rank1(&self, i: u32) -> u32 {
        let (hi, lo) = i.split();
        let mut rank = 0;
        for block in &self.blocks {
            if block.slot > hi {
                break;
            } else if block.slot == hi {
                rank += u32::from(block.repr.rank1(lo));
                break;
            } else {
                rank += block.repr.count1();
            }
        }
        rank
    }
}

impl Select1<u32> for Set {
    /// # Examples
    ///
    /// ```rust
    /// #[macro_use]
    /// extern crate compacts;
    /// fn main() {
    ///     use compacts::bits::Select1;
    ///     let bits = bits![0, 1, 4, 1 << 8, 1 << 16];
    ///     assert_eq!(bits.select1(0), Some(0));
    ///     assert_eq!(bits.select1(1), Some(1));
    ///     assert_eq!(bits.select1(2), Some(4));
    ///     assert_eq!(bits.select1(3), Some(1 << 8));
    /// }
    /// ```
    fn select1(&self, c: u32) -> Option<u32> {
        if self.count1() <= u64::from(c) {
            return None;
        }
        let mut remain = c;
        for block in &self.blocks {
            let w = block.repr.count1();
            if remain >= w {
                remain -= w;
            } else {
                let s = u32::from(block.repr.select1(remain as u16).unwrap());
                let k = u32::from(block.slot) << 16;
                return Some(s + k);
            }
        }
        None
    }
}

impl Select0<u32> for Set {
    /// # Examples
    ///
    /// ```rust
    /// #[macro_use]
    /// extern crate compacts;
    /// fn main() {
    ///     use compacts::bits::Select0;
    ///     let bits = bits![0, 1, 4, 1 << 8, 1 << 16];
    ///     assert_eq!(bits.select0(0), Some(2));
    ///     assert_eq!(bits.select0(1), Some(3));
    ///     assert_eq!(bits.select0(2), Some(5));
    ///     assert_eq!(bits.select0(3), Some(6));
    /// }
    /// ```
    fn select0(&self, c: u32) -> Option<u32> {
        if self.count0() <= u64::from(c) {
            return None;
        }
        select_by_rank!(0, self, c, 0u64, 1 << 32, u32)
    }
}

impl Block {
    fn slot(&self) -> u16 {
        self.slot
    }
    fn repr(&self) -> &Repr {
        &self.repr
    }
}

const SERIAL_COOKIE: u16 = 12_347; // `Seq16`, `Arr64` and `Run16`
const SERIAL_NO_RUN: u32 = 12_346; // `Seq16` and `Arr64`

// The cookie header spans either 32 bits or 64 bits.
//
// If the first 32 bits have the value `SERIAL_NO_RUN`,
// then there is no `Run16` block in `Map`.
// the next 32 bits are used to store the number of blocks.
// If the bitmap is empty (i.e., it has no container),
// then you should choose this cookie header.
//
// If the 16 least significant bits of the 32-bit cookie have the value `SERIAL_COOKIE`,
// the 16 most significant bits of the 32-bit cookie are used to store
// the number of blocks minus 1.
// That is, if you shift right by 16 the cookie and add 1, you get the number of blocks.
//
// Then we store `RunIndex` following the initial 32 bits,
// as a bitset to indicate whether each of the blocks is a `Run16` or not.
//
// The LSB of the first byte corresponds to the first stored blocks and so forth.
// Thus if follows that the least significant 16 bits of the first 32 bits
// of a serialized bitmaps should either have the value `SERIAL_NO_RUN`
// or the value SERIAL_COOKIE. In other cases, we should abort the decoding.
//
// After scanning the cookie header, we know how many containers are present in the bitmap.

const REPR16_CAPACITY: usize = 1 << 16;
const NO_OFFSET_THRESHOLD: u8 = 4;

struct RunIndex {
    hasrun: bool,
    bitmap: Vec<u8>,
}
impl fmt::Debug for RunIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "RunIndex(")?;
        for byte in &self.bitmap {
            write!(f, "{:08b}", byte)?;
        }
        write!(f, ")")?;
        Ok(())
    }
}

impl RunIndex {
    fn empty(&self) -> bool {
        !self.hasrun
    }
    fn bytes(&self) -> &[u8] {
        &self.bitmap
    }
}

impl Set {
    fn run_index(&self) -> RunIndex {
        let mut hasrun = false;
        let mut bitmap = vec![0u8; (self.blocks.len() + 7) / 8];
        for (i, b) in self.blocks.iter().enumerate() {
            if let Repr::Run(_) = b.repr {
                hasrun = true;
                bitmap[i / 8] |= 1 << (i % 8);
            }
        }
        RunIndex { hasrun, bitmap }
    }
}

impl<W: io::Write> WriteTo<W> for Set {
    fn write_to(&self, w: &mut W) -> io::Result<()> {
        let runidx = self.run_index();

        let (sizeof_cookie, sizeof_runidx) = if runidx.empty() {
            (2 * mem::size_of::<u32>(), 0)
        } else {
            (2 * mem::size_of::<u16>(), runidx.bytes().len())
        };

        let sizeof_header = 2 * mem::size_of::<u16>() * self.blocks.len();
        let sum_sizeof = sizeof_cookie + sizeof_runidx + sizeof_header;

        // serial cookie
        if runidx.empty() {
            SERIAL_NO_RUN.write_to(w)?;
            (self.blocks.len() as u32).write_to(w)?;
        } else {
            SERIAL_COOKIE.write_to(w)?;
            ((self.blocks.len() - 1) as u16).write_to(w)?;

            w.write_all(runidx.bytes())?;
        };

        // header
        for b in &self.blocks {
            let weight = (b.repr.count1() - 1) as u16;
            b.slot.write_to(w)?;
            weight.write_to(w)?;
        }

        if runidx.empty() || self.blocks.len() >= NO_OFFSET_THRESHOLD as usize {
            // offset
            let mut offset = sum_sizeof + 2 * mem::size_of::<u16>() * self.blocks.len();
            for b in &self.blocks {
                (offset as u32).write_to(w)?;
                let pop = b.repr.count1();
                match b.repr {
                    Repr::Seq(_) => {
                        assert!(pop as usize <= SEQ_MAX_LEN);
                        offset += mem::size_of::<u16>() * pop as usize;
                    }
                    Repr::Arr(_) => {
                        assert!(pop as usize > SEQ_MAX_LEN);
                        offset += REPR16_CAPACITY / 8;
                    }
                    Repr::Run(ref run) => {
                        offset += mem::size_of::<u16>();
                        offset += 2 * mem::size_of::<u16>() * run.ranges.len();
                    }
                }
            }
        }

        // TODO: Fix Block's WriteTo implementation
        // Write an optimized block (clone if it should do so),
        // so that the above assertions can be removed.

        for b in &self.blocks {
            match b.repr {
                Repr::Seq(ref seq) => seq.write_to(w)?,
                Repr::Arr(ref arr) => arr.write_to(w)?,
                Repr::Run(ref run) => run.write_to(w)?,
            }
        }

        Ok(())
    }
}

fn read_header<R: io::Read>(r: &mut R, size: usize) -> io::Result<Vec<(u16, u32)>> {
    let mut vec = Vec::with_capacity(size);
    for _ in 0..size {
        let key = read_from::<R, u16>(r)?;
        let pop = read_from::<R, u16>(r)?;
        vec.push((key, u32::from(pop) + 1));
    }
    // vec is sorted?
    Ok(vec)
}

fn discard_offset<R: io::Read>(r: &mut R, size: usize) -> io::Result<()> {
    let mut _offset = 0u32;
    for _ in 0..size {
        _offset.read_from(r)?;
    }
    Ok(())
}

impl<R: io::Read> ReadFrom<R> for Set {
    fn read_from(&mut self, r: &mut R) -> io::Result<()> {
        self.clear();

        match read_from::<R, u32>(r)? {
            cookie if cookie == SERIAL_NO_RUN => {
                let block_len = read_from::<R, u32>(r)? as usize;
                let header = read_header(r, block_len)?;

                // eprintln!("blocks={:?}", block_len);
                // eprintln!("header={:?}", header);
                discard_offset(r, block_len)?;

                for (slot, pop) in header {
                    let pop = pop as usize;
                    let repr = if pop > SEQ_MAX_LEN {
                        let arr = read_from::<R, Arr64>(r)?;
                        Repr::from(arr)
                    } else {
                        let mut seq = Seq16 {
                            vector: vec![0; pop],
                        };
                        seq.read_from(r)?;
                        Repr::from(seq)
                    };
                    self.blocks.push(Block { slot, repr });
                }
                Ok(())
            }

            cookie if cookie & 0x_0000_FFFF == u32::from(SERIAL_COOKIE) => {
                let block_len = (cookie.wrapping_shr(16) + 1) as usize;
                let bytes_len = (block_len + 7) / 8;

                let hasrun = true;
                let bitmap = {
                    let mut buf = vec![0; bytes_len];
                    r.read_exact(&mut buf)?;
                    buf
                };
                let runidx = RunIndex { hasrun, bitmap };
                let header = read_header(r, block_len)?;

                if runidx.empty() || block_len >= NO_OFFSET_THRESHOLD as usize {
                    discard_offset(r, block_len)?;
                }

                for (i, (slot, pop)) in header.into_iter().enumerate() {
                    let pop = pop as usize;

                    let repr = if runidx.bitmap[i / 8] & (1 << (i % 8)) > 0 {
                        let run = read_from::<R, Run16>(r)?;
                        Repr::from(run)
                    } else if pop > SEQ_MAX_LEN {
                        let arr = read_from::<R, Arr64>(r)?;
                        Repr::from(arr)
                    } else {
                        let mut seq = Seq16 {
                            vector: vec![0; pop],
                        };
                        seq.read_from(r)?;
                        Repr::from(seq)
                    };
                    self.blocks.push(Block { slot, repr });
                }
                Ok(())
            }

            x => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("unexpected cookie value: {}", x),
            )),
        }
    }
}

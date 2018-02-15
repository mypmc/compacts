#[macro_use]
mod macros;
mod dict;
mod repr;
mod pair;
mod iter;
#[cfg(test)]
mod tests;

use std::{io, mem, ops};
use std::ops::{BitAndAssign, BitOrAssign, BitXorAssign};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

trait BitAndNotAssign<RHS = Self> {
    fn bitandnot_assign(&mut self, that: RHS);
}

use self::repr::{ArrBlock, RunBlock, SeqBlock};
use self::repr::Block;
use self::repr::SEQ_MAX_LEN;

pub use self::dict::{PopCount, Rank, Select0, Select1};
pub use self::iter::{And, AndNot, Or, Xor};
pub use self::iter::{and, and_not, or, xor};

// https://www.cs.cmu.edu/~dga/papers/zhou-sea2013.pdf

/// Set of bit (2^32).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Set {
    slots: Vec<Slot>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct Slot {
    key: u16,
    bits: Block,
}

#[inline]
fn split(n: u32) -> (u16, u16) {
    let q = n / (1 << 16);
    let r = n % (1 << 16);
    (q as u16, r as u16)
}

#[inline]
fn merge(n: u16, m: u16) -> u32 {
    u32::from(n) * (1 << 16) + u32::from(m)
}

fn search_slot(slots: &[Slot], n: u16) -> Result<usize, usize> {
    slots.binary_search_by_key(&n, |slot| slot.key)
}

static TRUE: bool = true;
static FALSE: bool = false;

impl Set {
    /// Return new Set.
    pub fn new() -> Self {
        Set { slots: Vec::new() }
    }

    /// Clear all bits.
    ///
    /// # Examples
    ///
    /// ```rust
    /// #[macro_use]
    /// extern crate compacts;
    /// fn main() {
    ///     use compacts::bits::PopCount;
    ///     let mut bits = bitset!(0);
    ///     assert!(bits.count1() == 1);
    ///     bits.clear();
    ///     assert!(bits.count1() == 0);
    /// }
    /// ```
    pub fn clear(&mut self) {
        self.slots.clear();
    }

    /// Return **true** if the bit is enabled, **false** otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compacts::bits::{Set, PopCount};
    ///
    /// let mut bits = Set::new();
    /// assert_eq!(bits.count0(), 1 << 32);
    /// bits.put(1, true);
    /// assert!(!bits.get(0));
    /// assert!(bits.get(1));
    /// assert!(!bits.get(2));
    /// assert_eq!(bits.count1(), 1);
    /// ```
    #[inline]
    pub fn get(&self, x: u32) -> bool {
        let (key, bit) = split(x);
        search_slot(&self.slots, key)
            .map(|i| self.slots[i].bits.contains(bit))
            .unwrap_or(false)
    }

    /// Update bit at `x`, return **previous** value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// #[macro_use]
    /// extern crate compacts;
    /// fn main() {
    ///     let mut bits = bitset![1, 2, 8];
    ///     assert!(!bits.put(0, false));
    ///     assert!(bits.put(1, false));
    ///     assert!(!bits.put(1, true));
    ///     assert!(bits.put(1, true));
    /// }
    /// ```
    #[inline]
    pub fn put(&mut self, x: u32, enabled: bool) -> bool {
        if enabled {
            self.insert(x)
        } else {
            self.remove(x)
        }
    }

    /// Enable bit at `x`, return **previous** value.
    #[inline]
    pub fn insert(&mut self, x: u32) -> bool {
        let (key, bit) = split(x);
        let pos = search_slot(&self.slots, key);
        match pos {
            Ok(i) => {
                let slot = &mut self.slots[i];
                slot.bits.insert(bit)
            }
            Err(i) => {
                let mut bits = Block::new();
                bits.insert(bit);
                self.slots.insert(i, Slot { key, bits });
                false
            }
        }
    }

    /// Disable bit at `x`, return **previous** value.
    #[inline]
    pub fn remove(&mut self, x: u32) -> bool {
        let (slot, bit) = split(x);
        if let Ok(i) = search_slot(&self.slots, slot) {
            let slot = &mut self.slots[i];
            slot.bits.remove(bit)
        } else {
            false
        }
    }

    /// Optimize an innternal representaions.
    pub fn optimize(&mut self) {
        for slot in &mut self.slots {
            slot.bits.optimize();
        }
        self.slots.retain(|slot| slot.bits.count1() > 0);
        self.slots.shrink_to_fit();
    }

    pub fn bits<'a>(&'a self) -> impl Iterator<Item = u32> + 'a {
        self.slots.iter().flat_map(|slot| {
            let key = slot.key;
            slot.bits.iter().map(move |r| merge(key, r))
        })
    }
}

impl ops::Index<u32> for Set {
    type Output = bool;
    fn index(&self, i: u32) -> &Self::Output {
        if self.get(i) {
            &TRUE
        } else {
            &FALSE
        }
    }
}

impl<T: AsRef<[u32]>> From<T> for Set {
    fn from(v: T) -> Self {
        v.as_ref().iter().collect()
    }
}

impl<'a> ::std::iter::FromIterator<u32> for Set {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = u32>,
    {
        let mut bits = Set::new();
        for bit in iter {
            bits.insert(bit);
        }
        bits.optimize();
        bits
    }
}

impl<'a> ::std::iter::FromIterator<&'a u32> for Set {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a u32>,
    {
        let mut bits = Set::new();
        for &bit in iter {
            bits.insert(bit);
        }
        bits.optimize();
        bits
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
    ///     let bits = bitset![0, 1, 4, 1 << 8, 1 << 16];
    ///     assert_eq!(bits.count1(), 5);
    /// }
    /// ```
    fn count1(&self) -> u64 {
        self.slots.iter().map(|b| u64::from(b.bits.count1())).sum()
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
    ///     let bits = bitset![0, 1, 4, 1 << 8, 1 << 16];
    ///     assert_eq!(bits.rank1(0), 0);
    ///     assert_eq!(bits.rank1(1), 1);
    ///     assert_eq!(bits.rank1(2), 2);
    ///     assert_eq!(bits.rank1(3), 2);
    ///     assert_eq!(bits.rank1(4), 2);
    ///     assert_eq!(bits.rank1(5), 3);
    /// }
    /// ```
    fn rank1(&self, i: u32) -> u32 {
        let (hi, lo) = split(i);
        let mut rank = 0;
        for slot in &self.slots {
            if slot.key > hi {
                break;
            } else if slot.key == hi {
                rank += u32::from(slot.bits.rank1(lo));
                break;
            } else {
                rank += slot.bits.count1();
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
    ///     let bits = bitset![0, 1, 4, 1 << 8, 1 << 16];
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
        for slot in &self.slots {
            let w = slot.bits.count1();
            if remain >= w {
                remain -= w;
            } else {
                let s = u32::from(slot.bits.select1(remain as u16).unwrap());
                let k = u32::from(slot.key) << 16;
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
    ///     let bits = bitset![0, 1, 4, 1 << 8, 1 << 16];
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

impl Set {
    // The cookie header spans either 32 bits or 64 bits.
    //
    // If the first 32 bits have the value `SERIAL_NO_RUN`,
    // then there is no `RunBlock` slot in `Map`.
    // the next 32 bits are used to store the number of slots.
    // If the bitmap is empty (i.e., it has no container),
    // then you should choose this cookie header.
    //
    // If the 16 least significant bits of the 32-bit cookie have the value `SERIAL_COOKIE`,
    // the 16 most significant bits of the 32-bit cookie are used to store
    // the number of slots minus 1.
    // That is, if you shift right by 16 the cookie and add 1, you get the number of slots.
    //
    // Then we store `RunIndex` following the initial 32 bits,
    // as a bitset to indicate whether each of the slots is a `RunBlock` or not.
    //
    // The LSB of the first byte corresponds to the first stored slots and so forth.
    // Thus if follows that the least significant 16 bits of the first 32 bits
    // of a serialized bitmaps should either have the value `SERIAL_NO_RUN`
    // or the value SERIAL_COOKIE. In other cases, we should abort the decoding.
    //
    // After scanning the cookie header, we know how many containers are present in the bitmap.

    pub fn write_to<W: io::Write>(&self, w: &mut W) -> io::Result<()> {
        let runidx = RunIndex::new(&self.slots);

        let (sizeof_cookie, sizeof_runidx) = if runidx.is_empty() {
            (2 * mem::size_of::<u32>(), 0)
        } else {
            (2 * mem::size_of::<u16>(), runidx.bytes().len())
        };

        let sizeof_header = 2 * mem::size_of::<u16>() * self.slots.len();
        let sum_sizeof = sizeof_cookie + sizeof_runidx + sizeof_header;

        // serial cookie
        if runidx.is_empty() {
            w.write_u32::<LittleEndian>(SERIAL_NO_RUN)?;
            w.write_u32::<LittleEndian>(self.slots.len() as u32)?;
        } else {
            w.write_u16::<LittleEndian>(SERIAL_COOKIE)?;
            w.write_u16::<LittleEndian>((self.slots.len() - 1) as u16)?;
            w.write_all(runidx.bytes())?;
        };

        // header
        for slot in &self.slots {
            let weight = (slot.bits.count1() - 1) as u16;
            w.write_u16::<LittleEndian>(slot.key)?;
            w.write_u16::<LittleEndian>(weight)?;
        }

        if runidx.is_empty() || self.slots.len() >= NO_OFFSET_THRESHOLD as usize {
            // offset
            let mut offset = sum_sizeof + 2 * mem::size_of::<u16>() * self.slots.len();
            for b in &self.slots {
                w.write_u32::<LittleEndian>(offset as u32)?;
                let pop = b.bits.count1();
                match b.bits {
                    Block::Seq(_) => {
                        assert!(pop as usize <= SEQ_MAX_LEN);
                        offset += mem::size_of::<u16>() * pop as usize;
                    }
                    Block::Arr(_) => {
                        assert!(pop as usize > SEQ_MAX_LEN);
                        offset += (1 << 16) / 8;
                    }
                    Block::Run(ref run) => {
                        offset += mem::size_of::<u16>();
                        offset += 2 * mem::size_of::<u16>() * run.ranges().len();
                    }
                }
            }
        }

        // TODO: Fix Slot's WriteTo implementation
        // Write an optimized slot (clone if it should do so),
        // so that the above assertions can be removed.

        for b in &self.slots {
            match b.bits {
                Block::Seq(ref seq) => seq.write_to(w)?,
                Block::Arr(ref arr) => arr.write_to(w)?,
                Block::Run(ref run) => run.write_to(w)?,
            }
        }

        Ok(())
    }

    pub fn read_from<R: io::Read>(r: &mut R) -> io::Result<Self> {
        match r.read_u32::<LittleEndian>()? {
            cookie if cookie == SERIAL_NO_RUN => {
                let slot_len = r.read_u32::<LittleEndian>()? as usize;
                let header = read_header(r, slot_len)?;

                // eprintln!("slots={:?}", slot_len);
                // eprintln!("header={:?}", header);
                discard_offset(r, slot_len)?;

                let mut slots = Vec::with_capacity(slot_len);

                for (key, pop) in header {
                    let pop = pop as usize;
                    let bits = if pop > SEQ_MAX_LEN {
                        let arr = ArrBlock::read_from(r)?;
                        Block::from(arr)
                    } else {
                        let seq = SeqBlock::read_from(r, pop)?;
                        Block::from(seq)
                    };
                    slots.push(Slot { key, bits });
                }
                Ok(Set { slots })
            }

            cookie if cookie & 0x_0000_FFFF == u32::from(SERIAL_COOKIE) => {
                let slot_len = (cookie.wrapping_shr(16) + 1) as usize;
                let bytes_len = (slot_len + 7) / 8;

                let hasrun = true;
                let bitmap = {
                    let mut buf = vec![0; bytes_len];
                    r.read_exact(&mut buf)?;
                    buf
                };
                let runidx = RunIndex { hasrun, bitmap };
                let header = read_header(r, slot_len)?;

                if runidx.is_empty() || slot_len >= NO_OFFSET_THRESHOLD as usize {
                    discard_offset(r, slot_len)?;
                }

                let mut slots = Vec::with_capacity(slot_len);

                for (i, (key, pop)) in header.into_iter().enumerate() {
                    let pop = pop as usize;

                    let bits = if runidx.bitmap[i / 8] & (1 << (i % 8)) > 0 {
                        let run = RunBlock::read_from(r)?;
                        Block::from(run)
                    } else if pop > SEQ_MAX_LEN {
                        let arr = ArrBlock::read_from(r)?;
                        Block::from(arr)
                    } else {
                        let seq = SeqBlock::read_from(r, pop)?;
                        Block::from(seq)
                    };

                    slots.push(Slot { key, bits });
                }
                Ok(Set { slots })
            }

            x => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("unexpected cookie value: {}", x),
            )),
        }
    }
}

const SERIAL_COOKIE: u16 = 12_347; // `SeqBlock`, `ArrBlock` and `RunBlock`
const SERIAL_NO_RUN: u32 = 12_346; // `SeqBlock` and `ArrBlock`
const NO_OFFSET_THRESHOLD: u8 = 4;

struct RunIndex {
    hasrun: bool,
    bitmap: Vec<u8>,
}

impl RunIndex {
    fn new(slots: &[Slot]) -> Self {
        let mut hasrun = false;
        let mut bitmap = vec![0u8; (slots.len() + 7) / 8];
        for (i, b) in slots.iter().enumerate() {
            if let Block::Run(_) = b.bits {
                hasrun = true;
                bitmap[i / 8] |= 1 << (i % 8);
            }
        }
        RunIndex { hasrun, bitmap }
    }

    fn is_empty(&self) -> bool {
        !self.hasrun
    }
    fn bytes(&self) -> &[u8] {
        &self.bitmap
    }
}

fn read_header<R: io::Read>(r: &mut R, size: usize) -> io::Result<Vec<(u16, u32)>> {
    let mut vec = Vec::with_capacity(size);
    for _ in 0..size {
        let key = r.read_u16::<LittleEndian>()?;
        let pop = r.read_u16::<LittleEndian>()?;
        vec.push((key, u32::from(pop) + 1));
    }
    // vec is sorted?
    Ok(vec)
}

fn discard_offset<R: io::Read>(r: &mut R, size: usize) -> io::Result<()> {
    for _ in 0..size {
        let _offset = r.read_u32::<LittleEndian>()?;
    }
    Ok(())
}

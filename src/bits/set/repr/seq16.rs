use std::{cmp, fmt, io};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use bits;
use io::{ReadFrom, WriteTo};
use super::{Arr64, Compare, Run16};

#[derive(Clone, Default, PartialEq, Eq)]
pub(crate) struct Seq16 {
    pub vector: Vec<u16>,
}
impl fmt::Debug for Seq16 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Seq16({:?})", self.vector.len())
    }
}

impl Seq16 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(cap: usize) -> Self {
        let bounded = if cap <= bits::SEQ_MAX_LEN {
            cap
        } else {
            bits::SEQ_MAX_LEN
        };
        let vector = Vec::with_capacity(bounded);
        Seq16 { vector }
    }

    #[inline]
    pub fn search(&self, bit: &u16) -> Result<usize, usize> {
        self.vector.binary_search(bit)
    }

    #[inline]
    pub fn contains(&self, bit: u16) -> bool {
        self.search(&bit).is_ok()
    }

    #[inline]
    pub fn insert(&mut self, bit: u16) -> bool {
        self.search(&bit)
            .map_err(|i| self.vector.insert(i, bit))
            .is_err()
    }

    #[inline]
    pub fn remove(&mut self, bit: u16) -> bool {
        self.search(&bit).map(|i| self.vector.remove(i)).is_ok()
    }
}

impl From<Arr64> for Seq16 {
    fn from(that: Arr64) -> Self {
        Seq16::from(&that)
    }
}
impl<'r> From<&'r Arr64> for Seq16 {
    fn from(that: &Arr64) -> Self {
        use std::u16;
        let mut vec16 = Seq16::with_capacity(that.weight as usize);
        let iter = that.boxarr.iter();
        for (i, w) in iter.cloned().enumerate().filter(|&(_, v)| v != 0) {
            for p in 0..64 {
                if w & (1 << p) != 0 {
                    let bit = i * 64 + p;
                    debug_assert!(bit <= u16::MAX as usize);
                    vec16.insert(bit as u16);
                }
            }
        }
        vec16
    }
}

impl From<Run16> for Seq16 {
    fn from(that: Run16) -> Self {
        Seq16::from(&that)
    }
}
impl<'r> From<&'r Run16> for Seq16 {
    fn from(that: &'r Run16) -> Self {
        let mut seq16 = Seq16::with_capacity(that.weight as usize);
        for range in &that.ranges {
            seq16.vector.extend(range.clone());
        }
        seq16
    }
}

impl From<Vec<u16>> for Seq16 {
    fn from(vector: Vec<u16>) -> Self {
        let mut vector = vector;
        vector.sort();
        vector.dedup();
        assert!(vector.len() <= bits::SEQ_MAX_LEN);
        Seq16 { vector }
    }
}

impl<'a> super::Assign<&'a Seq16> for Seq16 {
    fn and_assign(&mut self, seq16: &'a Seq16) {
        *self = {
            let data = Compare::and(&*self, seq16).filter_map(|tup| match tup {
                (Some(l), Some(r)) if l == r => Some(l),
                _ => None,
            });
            let mut seq16 = Seq16::with_capacity(cmp::min(self.vector.len(), seq16.vector.len()));
            for bit in data {
                seq16.insert(bit);
            }
            seq16
        };
    }

    fn or_assign(&mut self, seq16: &'a Seq16) {
        for &bit in &seq16.vector {
            self.insert(bit);
        }
    }

    fn and_not_assign(&mut self, seq16: &'a Seq16) {
        *self = {
            let data = Compare::and_not(&*self, seq16).filter_map(|tup| match tup {
                (Some(l), None) => Some(l),
                _ => None,
            });
            let mut seq16 = Seq16::with_capacity(self.vector.len());
            for bit in data {
                seq16.insert(bit);
            }
            seq16
        };
    }

    fn xor_assign(&mut self, seq16: &'a Seq16) {
        for &bit in &seq16.vector {
            if !self.insert(bit) {
                self.remove(bit);
            }
        }

        // for &bit in &seq16.vector {
        //     if self.contains(bit) {
        //         self.remove(bit);
        //     } else {
        //         self.insert(bit);
        //     }
        // }
    }
}

impl<W: io::Write> WriteTo<W> for Seq16 {
    fn write_to(&self, w: &mut W) -> io::Result<()> {
        for &bit in &self.vector {
            w.write_u16::<LittleEndian>(bit)?;
        }
        Ok(())
    }
}

impl<R: io::Read> ReadFrom<R> for Seq16 {
    // `self.vector` must have an enough length.
    fn read_from(&mut self, r: &mut R) -> io::Result<()> {
        for bit in &mut self.vector {
            *bit = r.read_u16::<LittleEndian>()?;
        }
        Ok(())
    }
}

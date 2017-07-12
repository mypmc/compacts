#[cfg(test)]
#[macro_use]
mod macros;

mod seq16;
mod seq64;
mod rle16;
mod range;
mod iter;

#[cfg(test)]
mod tests;

pub use self::iter::*;

use std::ops::RangeInclusive;
use std::mem;

pub(crate) const CAPACITY: usize = 1 << 16;
const CAP32: u32 = CAPACITY as u32;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Seq<T> {
    pub(crate) weight: u32,
    pub(crate) vector: Vec<T>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Rle<T> {
    pub(crate) weight: u32,
    pub(crate) ranges: Vec<RangeInclusive<T>>,
}

pub(crate) type Seq16 = Seq<u16>;
pub(crate) type Seq64 = Seq<u64>;
pub(crate) type Rle16 = Rle<u16>;

impl<T> Seq<T> {
    pub fn len(&self) -> usize {
        self.vector.len()
    }
    pub fn is_empty(&self) -> bool {
        self.weight == 0 && self.len() == 0
    }

    pub fn count_ones(&self) -> u32 {
        self.weight
    }
    pub fn count_zeros(&self) -> u32 {
        CAP32 - self.count_ones()
    }

    pub fn load_factor(&self) -> f64 {
        self.count_ones() as f64 / CAP32 as f64
    }

    pub fn shrink_to_fit(&mut self) {
        self.vector.shrink_to_fit()
    }
}

impl<T> Rle<T> {
    pub fn len(&self) -> usize {
        self.ranges.len()
    }
    pub fn is_empty(&self) -> bool {
        self.weight == 0 && self.len() == 0
    }

    pub fn count_ones(&self) -> u32 {
        self.weight
    }
    pub fn count_zeros(&self) -> u32 {
        CAP32 - self.count_ones()
    }

    pub fn load_factor(&self) -> f64 {
        self.count_ones() as f64 / CAP32 as f64
    }

    pub fn shrink_to_fit(&mut self) {
        self.ranges.shrink_to_fit()
    }
}

impl Seq16 {
    pub fn size_in_bytes(weight: usize) -> usize {
        weight * mem::size_of::<u16>() + mem::size_of::<u32>()
    }
    pub fn mem_size(&self) -> usize {
        Self::size_in_bytes(self.weight as usize)
    }
}
impl Seq64 {
    pub fn size_in_bytes(len: usize) -> usize {
        len * mem::size_of::<u64>() + mem::size_of::<u32>()
    }
    pub fn mem_size(&self) -> usize {
        // seq64 has fixed size
        Self::size_in_bytes(1024)
    }
}
impl Rle16 {
    pub fn size_in_bytes(runlen: usize) -> usize {
        runlen * mem::size_of::<RangeInclusive<u16>>() + mem::size_of::<u32>()
    }
    pub fn mem_size(&self) -> usize {
        Self::size_in_bytes(self.ranges.len())
    }
}

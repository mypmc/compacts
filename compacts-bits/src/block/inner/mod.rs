mod seq16;
mod seq64;
mod rle16;
mod iter;
mod range;

#[cfg(test)]
mod tests;

pub use self::iter::*;

use std::ops::RangeInclusive;
use std::mem;
use std::fmt;

pub const CAPACITY: usize = 1 << 16;

#[derive(Clone, PartialEq)]
pub struct Seq<T> {
    pub weight: u32,
    pub vector: Vec<T>,
}

#[derive(Clone, PartialEq)]
pub struct Rle<T> {
    pub weight: u32,
    pub ranges: Vec<RangeInclusive<T>>,
}

pub type Seq16 = Seq<u16>;
pub type Seq64 = Seq<u64>;
pub type Rle16 = Rle<u16>;

impl<T> Seq<T> {
    pub fn mem(&self) -> usize {
        mem::size_of::<u32>() + mem::size_of::<T>() * self.vector.len()
    }
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
        super::Block::CAPACITY - self.count_ones()
    }

    pub fn load_factor(&self) -> f64 {
        self.count_ones() as f64 / super::Block::CAPACITY as f64
    }

    pub fn shrink_to_fit(&mut self) {
        self.vector.shrink_to_fit()
    }
}

impl<T> Rle<T> {
    pub fn mem(&self) -> usize {
        mem::size_of::<u32>() + mem::size_of::<RangeInclusive<T>>() * self.ranges.len()
    }
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
        super::Block::CAPACITY - self.count_ones()
    }

    pub fn load_factor(&self) -> f64 {
        self.count_ones() as f64 / super::Block::CAPACITY as f64
    }

    pub fn shrink_to_fit(&mut self) {
        self.ranges.shrink_to_fit()
    }
}

const UNIT: f64 = 1024.0;

impl fmt::Debug for Seq<u16> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let m = self.mem() as f64 / UNIT;
        let l = self.load_factor();
        write!(f, "seq16({:4.1}(kb) {:4.2})", m, l)
    }
}
impl fmt::Debug for Seq<u64> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let m = self.mem() as f64 / UNIT;
        let l = self.load_factor();
        write!(f, "seq64({:4.1}(kb) {:4.2})", m, l)
    }
}
impl fmt::Debug for Rle<u16> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let m = self.mem() as f64 / UNIT;
        let l = self.load_factor();
        write!(f, "rle16({:4.1}(kb) {:4.2})", m, l)
    }
}

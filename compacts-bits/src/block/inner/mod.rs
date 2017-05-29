mod seq16;
mod seq64;
mod rle16;
mod iter;
mod range;
#[cfg(test)]
mod tests;

pub use self::iter::*;

pub const CAPACITY: usize = 1 << 16;

#[derive(Debug, Clone, PartialEq)]
pub struct Seq<T> {
    pub weight: u32,
    pub vector: Vec<T>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Rle<T> {
    pub weight: u32,
    pub ranges: Vec<::std::ops::RangeInclusive<T>>,
}

pub type Seq16 = Seq<u16>;
pub type Seq64 = Seq<u64>;
pub type Rle16 = Rle<u16>;

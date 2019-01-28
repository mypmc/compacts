//! Module bits defines traits and types to interact with a bits container.
//!
//! ## `Map<T>` and `[T]`
//!
//! ## `PageMap<K, V>` and `[Page<K, V>]`

// # References
//
// - Compact Data Structures: A Practical Approach
// - Fast, Small, Simple Rank/Select on Bitmaps
// - Space-Efficient, High-Performance Rank & Select Structures on Uncompressed Bit Sequences

#[macro_use]
mod macros;
#[cfg(test)]
mod tests;

mod mask;
mod page;
mod uint;

// mod range;

#[allow(dead_code)]
pub mod rrr15 {
    generate_rrr_mod!("/table15.rs", u16, 15, 4);
}
#[allow(dead_code)]
pub mod rrr31 {
    generate_rrr_mod!("/table31.rs", u32, 31, 5);
}
#[allow(dead_code)]
pub mod rrr63 {
    generate_rrr_mod!("/table63.rs", u64, 63, 6);
}

use std::{
    borrow::Cow,
    ops::{Bound, Range, RangeBounds},
};

pub use self::{
    mask::{and, or, xor},
    mask::{Fold, Mask},
    page::{Page, PageMap},
    uint::Block,
};

use self::uint::{TryCastInto, UnsignedInt};

const MAX_BITS: u64 = 1 << 63;

// Panic message.
static OUT_OF_BOUNDS: &str = "index out of bounds";

/// `FiniteBits` denotes types with a finite, fixed number of bits.
///
/// This trait is for types intended to use as a component of the bits container.
/// e.g.) T of `Map<T>`, V of `PageMap<K, V>`
pub trait FiniteBits: Clone + Count {
    /// The potential bit size.
    ///
    /// This constant value corresponds to total of enabled/disabled bits.
    const BITS: u64;

    /// Returns an empty bits container.
    ///
    /// The number of disabled bits of an empty instance must be equal to `BITS`.
    fn empty() -> Self;
}

/// `Count` is a trait that counts the number of enabled/disabled bits in the container.
///
/// Every method have a cycled default implementations.
/// At least two methods need be re-defined.
pub trait Count {
    /// The value corresponds to total of enabled/disabled bits.
    /// Defined as `count1 + count0`.
    fn bits(&self) -> u64 {
        self.count1() + self.count0()
    }

    /// Return the number of enabled bits in the container.
    /// Defined as `bits - count0`.
    ///
    /// Counting bits is not always `O(1)`. It depends on the implementation.
    fn count1(&self) -> u64 {
        self.bits() - self.count0()
    }

    /// Return the number of disabled bits in the container.
    /// Defined as `bits - count1`.
    ///
    /// Counting bits is not always `O(1)`. It depends on the implementation.
    fn count0(&self) -> u64 {
        self.bits() - self.count1()
    }
}

/// `Access` is a trait to test bit.
pub trait Access {
    fn access(&self, index: u64) -> bool;

    /// Return the positions of all enabled bits in the container.
    ///
    /// Default implementation is just a accessing to all bits.
    ///
    /// ```
    /// use compacts::bits::Access;
    /// let word = [0b_10101010_u8, 0b_11110000_u8];
    /// let bits = word.iterate().collect::<Vec<_>>();
    /// assert_eq!(bits, vec![1, 3, 5, 7, 12, 13, 14, 15]);
    /// ```
    fn iterate<'a>(&'a self) -> Box<dyn Iterator<Item = u64> + 'a>
    where
        Self: Count,
    {
        Box::new((0..self.bits()).filter_map(move |i| if self.access(i) { Some(i) } else { None }))
    }
}

/// `Assign` is a trait to enable/disable bits.
pub trait Assign<Idx> {
    type Output;
    fn set1(&mut self, index: Idx) -> Self::Output;
    fn set0(&mut self, index: Idx) -> Self::Output;
}

#[doc(hidden)]
pub enum Excess {
    Rank1(u64), // rank1 > rank0
    Rank0(u64), // rank1 < rank0
}

/// `Rank` is a generization of `Count`.
///
/// Both `rank1` and `rank0` have default implementation, but these are cycled.
/// Either `rank1` or `rank0` need to be re-defined.
pub trait Rank: Count {
    /// Returns the number of enabled bits in `[0, i)`.
    /// Defined as `i - rank0`.
    ///
    /// `rank1(self.bits())` is equal to `count1()`.
    ///
    /// # Panics
    ///
    /// Panics if `i > bits`.
    fn rank1(&self, i: u64) -> u64 {
        assert!(i <= self.bits(), OUT_OF_BOUNDS);
        i - self.rank0(i)
    }

    /// Returns the number of disabled bits in `[0, i)`.
    /// Difined as `i - rank1`.
    ///
    /// `rank0(self.bits())` is equal to `count0()`.
    ///
    /// # Panics
    ///
    /// Panics if `i > bits`.
    fn rank0(&self, i: u64) -> u64 {
        assert!(i <= self.bits(), OUT_OF_BOUNDS);
        i - self.rank1(i)
    }

    /// Searches the position of `n+1`th enabled bit by binary search.
    #[doc(hidden)]
    fn search1(&self, n: u64) -> Option<u64> {
        if n < self.count1() {
            Some(search_index(self.bits(), |k| self.rank1(k) > n) - 1)
        } else {
            None
        }
    }

    /// Searches the position of `n+1`th disabled bit by binary search.
    #[doc(hidden)]
    fn search0(&self, n: u64) -> Option<u64> {
        if n < self.count0() {
            Some(search_index(self.bits(), |k| self.rank0(k) > n) - 1)
        } else {
            None
        }
    }

    /// Returns an excess of rank.
    #[doc(hidden)]
    fn excess(&self, i: u64) -> Option<Excess> {
        use std::cmp::Ordering::{Equal as EQ, Greater as GE, Less as LE};

        let rank1 = self.rank1(i);
        let rank0 = i - rank1;
        match rank1.cmp(&rank0) {
            EQ => None,
            LE => Some(Excess::Rank0(rank0 - rank1)),
            GE => Some(Excess::Rank1(rank1 - rank0)),
        }
    }
}

/// Right inverse of `rank1`.
pub trait Select1: Count {
    /// Returns the position of 'n+1'th occurences of `1`.
    fn select1(&self, n: u64) -> Option<u64>;
}

/// Right inverse of `rank0`.
pub trait Select0: Count {
    /// Returns the position of 'n+1'th occurences of `0`.
    fn select0(&self, n: u64) -> Option<u64>;
}

#[allow(clippy::range_plus_one)]
#[rustfmt::skip]
pub(crate) fn from_bounds<R: RangeBounds<u64>>(range: &'_ R, bits: u64) -> Range<u64> {
    use Bound::*;
    match (range.start_bound(), range.end_bound()) {

        (Included(&i), Included(&j)) if i   < bits && i <= j && j <  bits => i   .. j+1,
        (Included(&i), Excluded(&j)) if i   < bits && i <= j && j <= bits => i   .. j,
        (Excluded(&i), Included(&j)) if i+1 < bits && i <  j && j <  bits => i+1 .. j+1,
        (Excluded(&i), Excluded(&j)) if i+1 < bits && i <  j && j <= bits => i+1 .. j,

        // i == 0
        (Unbounded, Included(&j)) if j <  bits => 0 .. j+1,
        (Unbounded, Excluded(&j)) if j <= bits => 0 .. j,

        // j == bits
        (Included(&i), Unbounded) if i   < bits => i   .. bits,
        (Excluded(&i), Unbounded) if i+1 < bits => i+1 .. bits,

        (Unbounded, Unbounded) => 0 .. bits,

        _ => panic!("unexpected range"),
    }
}

impl<'a, T: ?Sized + Count + Assign<U>, U: RangeBounds<u64> + Clone> Assign<&'a U> for T {
    type Output = <T as Assign<U>>::Output;
    fn set1(&mut self, r: &'a U) -> Self::Output {
        self.set1(r.clone())
    }
    fn set0(&mut self, r: &'a U) -> Self::Output {
        self.set0(r.clone())
    }
}

macro_rules! implsRangeBounds {
    ($($Type:ty),*) => ($(
        impl<T: ?Sized + Count + Assign<Range<u64>>> Assign<$Type> for T {
            type Output = <T as Assign<Range<u64>>>::Output;
            fn set1(&mut self, r: $Type) -> Self::Output {
                self.set1(from_bounds(&r, self.bits()))
            }
            fn set0(&mut self, r: $Type) -> Self::Output {
                self.set0(from_bounds(&r, self.bits()))
            }
        }
    )*)
}
implsRangeBounds!(
    std::ops::RangeTo<u64>,
    std::ops::RangeFull,
    std::ops::RangeFrom<u64>,
    std::ops::RangeInclusive<u64>,
    std::ops::RangeToInclusive<u64>
);

/// Cast U into T.
///
/// # Panics
///
/// Panics if given `u` does not fit in `T`.
#[inline]
fn ucast<U, T>(u: U) -> T
where
    U: UnsignedInt + TryCastInto<T>,
    T: UnsignedInt,
{
    u.try_cast_into().expect("does not fit in T")
}

#[inline]
fn divmod<U: UnsignedInt>(i: u64, cap: u64) -> (U, u64)
where
    u64: TryCastInto<U>,
{
    (ucast(i / cap), i % cap)
}

/// Search the smallest index in range at which f(i) is true,
/// assuming that f(i) == true implies f(i+1) == true.
fn search_index<T: UnsignedInt>(k: T, func: impl Fn(T) -> bool) -> T {
    let mut i = 0;
    let mut j = ucast::<T, usize>(k);
    while i < j {
        let h = i + (j - i) / 2;
        if func(ucast(h)) {
            j = h; // f(j) == true
        } else {
            i = h + 1; // f(i-1) == false
        }
    }
    ucast(i) // f(i-1) == false && f(i) (= f(j)) == true
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Map<T> {
    ones: u64,
    data: Vec<T>,
}

impl<T> Default for Map<T> {
    fn default() -> Self {
        Map::new_unchecked(0, Vec::new())
    }
}

impl<T> AsRef<[T]> for Map<T> {
    fn as_ref(&self) -> &[T] {
        self.data.as_slice()
    }
}

impl<T> Map<T> {
    pub fn new() -> Self {
        Map::new_unchecked(0, Vec::new())
    }

    fn new_unchecked(ones: u64, data: Vec<T>) -> Self {
        Map { ones, data }
    }

    pub fn with<U: AsRef<[T]>>(slice: U) -> Map<T>
    where
        T: FiniteBits,
    {
        let ones = slice.as_ref().iter().fold(0, |acc, t| acc + t.count1());
        let data = slice.as_ref().to_vec();
        Map { ones, data }
    }

    /// Shrink an internal vector.
    pub fn shrink_to_fit(&mut self) {
        self.data.shrink_to_fit()
    }

    pub fn into_vec(self) -> Vec<T> {
        self.data
    }

    fn access<U: ?Sized>(data: &U, i: u64) -> bool
    where
        T: FiniteBits + Access,
        U: AsRef<[T]> + Count,
    {
        assert!(i < data.bits(), OUT_OF_BOUNDS);
        let (i, o) = divmod::<usize>(i, T::BITS);
        data.as_ref().get(i).map_or(false, |t| t.access(o))
    }

    fn rank1<U: ?Sized>(data: &U, i: u64) -> u64
    where
        T: FiniteBits + Rank,
        U: AsRef<[T]> + Count,
    {
        assert!(i <= data.bits(), OUT_OF_BOUNDS);
        let (i, o) = divmod(i, T::BITS);
        let slice = data.as_ref();
        let c = slice.iter().take(i).fold(0, |acc, b| acc + b.count1());
        let r = slice.get(i).map_or(0, |b| b.rank1(o));
        c + r
    }

    fn select1<U: ?Sized>(data: &U, mut n: u64) -> Option<u64>
    where
        T: FiniteBits + Select1,
        U: AsRef<[T]>,
    {
        for (k, v) in data.as_ref().iter().enumerate() {
            let count = v.count1();
            if n < count {
                let select1 = v.select1(n).expect("remain < count");
                return Some(ucast::<usize, u64>(k) * T::BITS + select1);
            }
            n -= count;
        }
        None
    }
}

impl<T> Count for Map<T>
where
    T: FiniteBits,
{
    /// # Examples
    ///
    /// ```
    /// use compacts::bits::{Map, Count};
    /// let map = Map::with([0u64, 0b10101100000, 0b0000100000]);
    /// assert_eq!(1<<63, map.bits());
    /// assert_eq!(192,   map.as_ref().bits());
    /// ```
    fn bits(&self) -> u64 {
        MAX_BITS
    }

    /// # Examples
    ///
    /// ```
    /// use compacts::bits::{Map, Count};
    /// let map = Map::with([0u64, 0b10101100000, 0b0000100000]);
    /// assert_eq!(map.count1(), 5);
    /// assert_eq!(map.count1(), map.as_ref().count1());
    /// ```
    fn count1(&self) -> u64 {
        debug_assert!(self.ones <= self.bits());
        self.ones
    }
}
impl<T> Count for [T]
where
    T: FiniteBits,
{
    fn bits(&self) -> u64 {
        ucast::<usize, u64>(self.len()) * T::BITS
    }
    fn count1(&self) -> u64 {
        self.iter().fold(0, |acc, w| acc + w.count1())
    }
}

impl<T> Access for Map<T>
where
    T: FiniteBits + Access,
{
    /// Test bit at a given position.
    ///
    /// # Examples
    ///
    /// ```
    /// use compacts::bits::{Map, Access};
    /// let map = Map::with([0b_00000101u64, 0b01100011]);
    /// assert!( map.access(0));
    /// assert!(!map.access(1));
    /// assert!( map.access(2));
    /// assert!(!map.access(16));
    /// ```
    ///
    /// The length of slice must be greater than `i % T::BITS`.
    ///
    /// ```
    /// # use compacts::bits::{Map, Access};
    /// # let map = Map::with([0b_00000101u64, 0b01100011]);
    /// let slice = map.as_ref();
    /// assert!( slice.access(0));
    /// assert!(!slice.access(1));
    /// assert!( slice.access(2));
    /// // this will panic
    /// // assert!(!slice.access(16));
    /// ```
    ///
    /// Slicing constructs another slice of bits.
    ///
    /// ```
    /// # use compacts::bits::{Map, Access};
    /// # let map = Map::with([0b_00000101u64, 0b01100011]);
    /// # let slice = map.as_ref();
    /// let slice = &slice[1..];
    /// assert!( slice.access(0));
    /// assert!( slice.access(1));
    /// assert!(!slice.access(2));
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `i >= self.bits()`.
    fn access(&self, i: u64) -> bool {
        Map::access(self, i)
    }

    /// Return the positions of all enabled bits in the container.
    ///
    /// ```
    /// use compacts::bits::Access;
    /// let word = [0b_10101010_u8, 0b_11110000_u8];
    /// let bits = word.iterate().collect::<Vec<_>>();
    /// assert_eq!(bits, vec![1, 3, 5, 7, 12, 13, 14, 15]);
    /// ```
    fn iterate<'a>(&'a self) -> Box<dyn Iterator<Item = u64> + 'a> {
        Box::new(self.data.iter().enumerate().flat_map(|(i, t)| {
            let offset = ucast::<usize, u64>(i) * T::BITS;
            t.iterate().map(move |j| j + offset)
        }))
    }
}
impl<T> Access for [T]
where
    T: FiniteBits + Access,
{
    fn access(&self, i: u64) -> bool {
        Map::access(self, i)
    }

    /// Return the positions of all enabled bits in the container.
    ///
    /// ```
    /// use compacts::bits::Access;
    /// let word = [0b_10101010_u8, 0b_11110000_u8];
    /// let bits = word.iterate().collect::<Vec<_>>();
    /// assert_eq!(bits, vec![1, 3, 5, 7, 12, 13, 14, 15]);
    /// ```
    fn iterate<'a>(&'a self) -> Box<dyn Iterator<Item = u64> + 'a> {
        Box::new(self.iter().enumerate().flat_map(|(i, t)| {
            let offset = ucast::<usize, u64>(i) * T::BITS;
            t.iterate().map(move |j| j + offset)
        }))
    }
}

impl<T> Rank for Map<T>
where
    T: FiniteBits + Rank,
{
    /// Returns the number of enabled bits in `[0, i)`.
    ///
    /// The length of slice must be greater than `i % T::BITS`.
    ///
    /// ```
    /// use compacts::bits::{Count, Rank};
    /// let slice = [0b_00000000u8, 0b_01100000, 0b_00010000];
    /// assert_eq!(slice.rank1(10), 0);
    /// assert_eq!(slice.rank1(14), 1);
    /// assert_eq!(slice.rank1(15), 2);
    /// assert_eq!(slice.rank1(16), 2);
    /// assert_eq!(slice.rank1(slice.bits()), 3);
    ///
    /// let slice = &slice[1..]; // [0b_01100000, 0b_00010000]
    /// assert_eq!(slice.rank1(8), 2);
    /// assert_eq!(slice.rank1(15), 3);
    /// ```
    fn rank1(&self, i: u64) -> u64 {
        Map::rank1(self, i)
    }
}
impl<T> Rank for [T]
where
    T: FiniteBits + Rank,
{
    fn rank1(&self, i: u64) -> u64 {
        Map::rank1(self, i)
    }
}

impl<T> Select1 for Map<T>
where
    T: FiniteBits + Select1,
{
    /// # Examples
    ///
    /// ```
    /// use compacts::bits::{Map, Select1};
    /// let map = Map::with([0b_00000000_u8, 0b_01000000, 0b_00001001]);
    /// assert_eq!(map.select1(0), Some(14));
    /// assert_eq!(map.select1(1), Some(16));
    /// assert_eq!(map.select1(2), Some(19));
    /// assert_eq!(map.select1(3), None);
    /// ```
    ///
    /// ```
    /// # use compacts::bits::{Map, Select1};
    /// # let map = Map::with([0b_00000000_u8, 0b_01000000, 0b_00001001]);
    /// assert_eq!(map.as_ref().select1(0), Some(14));
    /// assert_eq!(map.as_ref().select1(1), Some(16));
    /// assert_eq!(map.as_ref().select1(2), Some(19));
    /// assert_eq!(map.as_ref().select1(3), None);
    /// ```
    fn select1(&self, n: u64) -> Option<u64> {
        Map::select1(self, n)
    }
}
impl<T> Select1 for [T]
where
    T: FiniteBits + Select1,
{
    fn select1(&self, n: u64) -> Option<u64> {
        Map::select1(self, n)
    }
}

impl<T> Select0 for Map<T>
where
    T: FiniteBits + Select0,
{
    /// # Examples
    ///
    /// ```
    /// use compacts::bits::{Map, Select0};
    /// let map = Map::with([0b_11110111_u8, 0b_11111110, 0b_10010011]);
    /// assert_eq!(map.select0(0), Some(3));
    /// assert_eq!(map.select0(1), Some(8));
    /// assert_eq!(map.select0(2), Some(18));
    /// assert_eq!(map.select0(6), Some(24));
    /// ```
    /// ```
    /// # use compacts::bits::{Map, Select0};
    /// # let map = Map::with([0b_11110111_u8, 0b_11111110, 0b_10010011]);
    /// let slice = map.as_ref();
    /// assert_eq!(slice.select0(0), Some(3));
    /// assert_eq!(slice.select0(1), Some(8));
    /// assert_eq!(slice.select0(2), Some(18));
    /// assert_eq!(slice.select0(6), None);
    /// ```
    fn select0(&self, mut n: u64) -> Option<u64> {
        for (k, v) in self.data.iter().enumerate() {
            let count = v.count0();
            if n < count {
                let select0 = v.select0(n).expect("remain < count");
                return Some(ucast::<usize, u64>(k) * T::BITS + select0);
            }
            n -= count;
        }
        let select = ucast::<usize, u64>(self.data.len()) * T::BITS + n;
        if select < self.bits() {
            Some(select)
        } else {
            None
        }
    }
}
impl<T> Select0 for [T]
where
    T: FiniteBits + Select0,
{
    fn select0(&self, mut n: u64) -> Option<u64> {
        for (k, v) in self.iter().enumerate() {
            let count = v.count0();
            if n < count {
                let select0 = v.select0(n).expect("remain < count");
                return Some(ucast::<usize, u64>(k) * T::BITS + select0);
            }
            n -= count;
        }
        None
    }
}

impl<T> Assign<u64> for Map<T>
where
    T: FiniteBits + Access + Assign<u64>,
{
    type Output = ();

    /// Enable bit at a given position.
    ///
    /// # Examples
    ///
    /// ```
    /// use compacts::bits::{Map, Access, Assign};
    /// let mut map = Map::with([0u64, 0b10101100000, 0b0000100000]);
    /// map.set1(0);
    /// map.set1(2);
    /// assert!( map.access(0));
    /// assert!(!map.access(1));
    /// assert!( map.access(2));
    /// ```
    ///
    /// The length of slice must be greater than `i % T::BITS`.
    ///
    fn set1(&mut self, i: u64) -> Self::Output {
        assert!(i < self.bits(), OUT_OF_BOUNDS);
        let (i, o) = divmod::<usize>(i, T::BITS);

        if i >= self.data.len() {
            self.data.resize(i + 1, T::empty());
            self.ones += 1;
            self.data[i].set1(o);
        } else if !self.data[i].access(o) {
            self.ones += 1;
            self.data[i].set1(o);
        }
    }

    /// Disable bit at a given position.
    ///
    /// # Examples
    ///
    /// ```
    /// use compacts::bits::{Access, Assign};
    /// let mut slice = [0u64, 0b10101100001, 0b0000100000];
    /// assert!( slice.access(64));
    /// slice.set0(64);
    /// assert!(!slice.access(64));
    /// ```
    ///
    /// The length of slice must be greater than `i % T::BITS`.
    ///
    fn set0(&mut self, i: u64) -> Self::Output {
        assert!(i < self.bits(), OUT_OF_BOUNDS);
        let (i, o) = divmod::<usize>(i, T::BITS);

        if i < self.data.len() && self.data[i].access(o) {
            self.ones -= 1;
            self.data[i].set0(o);
        }
    }
}

impl<T> Assign<u64> for [T]
where
    T: FiniteBits + Assign<u64>,
{
    type Output = <T as Assign<u64>>::Output;

    fn set1(&mut self, i: u64) -> Self::Output {
        assert!(i < self.bits(), OUT_OF_BOUNDS);
        let (i, o) = divmod::<usize>(i, T::BITS);
        self[i].set1(o)
    }

    fn set0(&mut self, i: u64) -> Self::Output {
        assert!(i < self.bits(), OUT_OF_BOUNDS);
        let (i, o) = divmod::<usize>(i, T::BITS);
        self[i].set0(o)
    }
}

impl<T> Assign<Range<u64>> for Map<T>
where
    T: FiniteBits + Assign<Range<u64>, Output = u64>,
{
    type Output = u64;

    /// # Examples
    ///
    /// ```
    /// use compacts::bits::{Map, Assign};
    /// let mut map = Map::<u8>::new();
    /// assert_eq!(map.set1(0..3), 3);
    /// assert_eq!(map.as_ref(), [0b_00000111]);
    /// assert_eq!(map.set1(20..23), 3);
    /// assert_eq!(map.as_ref(), [0b_00000111, 0b_00000000, 0b_01110000]);
    /// assert_eq!(map.set1(20..28), 5);
    /// assert_eq!(map.as_ref(), [0b_00000111, 0b_00000000, 0b_11110000, 0b_00001111]);
    ///
    /// assert_eq!(map.set0(21..121), 7);
    /// assert_eq!(map.as_ref(), [0b_00000111, 0b_00000000, 0b_00010000]);
    /// assert_eq!(map.set0(20..21), 1);
    /// assert_eq!(map.as_ref(), [0b_00000111, 0b_00000000, 0b_00000000]);
    /// assert_eq!(map.set0(200..300), 0);
    /// assert_eq!(map.as_ref(), [0b_00000111, 0b_00000000, 0b_00000000]);
    /// assert_eq!(map.set0(2..102), 1);
    /// assert_eq!(map.as_ref(), [0b_00000011]);
    /// ```
    #[allow(clippy::range_plus_one)]
    fn set1(&mut self, r: Range<u64>) -> Self::Output {
        let prev = self.ones;
        self.ones += {
            if r.start >= r.end {
                0
            } else {
                let i = r.start;
                let j = r.end - 1;

                let (head_index, head_offset) = divmod::<usize>(i, T::BITS);
                let (last_index, last_offset) = divmod::<usize>(j, T::BITS);
                if head_index == last_index {
                    if head_index >= self.data.len() {
                        self.data.resize(head_index + 1, T::empty());
                    }

                    self.data[head_index].set1(head_offset..last_offset + 1)
                } else {
                    if last_index >= self.data.len() {
                        self.data.resize(last_index + 1, T::empty());
                    }

                    let mut out = 0;
                    out += self.data[head_index].set1(head_offset..T::BITS);
                    for i in (head_index + 1)..last_index {
                        out += self.data[i].set1(0..T::BITS);
                    }
                    out + self.data[last_index].set1(0..last_offset + 1)
                }
            }
        };
        self.ones - prev
    }

    #[allow(clippy::range_plus_one)]
    fn set0(&mut self, r: Range<u64>) -> Self::Output {
        let prev = self.ones;
        self.ones -= {
            if r.start >= r.end {
                0
            } else {
                let i = r.start;
                let j = r.end - 1;

                let (head_index, head_offset) = divmod::<usize>(i, T::BITS);
                let (last_index, last_offset) = divmod::<usize>(j, T::BITS);
                if self.data.len() <= head_index {
                    return 0;
                }
                if head_index == last_index {
                    self.data[head_index].set0(head_offset..last_offset + 1)
                } else if last_index < self.data.len() {
                    // head_index < self.len() && last_index < self.len()
                    let mut out = 0;
                    out += self.data[head_index].set0(head_offset..T::BITS);
                    for i in (head_index + 1)..last_index {
                        out += self.data[i].set0(0..T::BITS);
                    }
                    out + self.data[last_index].set0(0..last_offset + 1)
                } else {
                    // head_index < self.len() && self.len() <= last_index
                    let mut out = self.data[head_index].set0(head_offset..T::BITS);
                    out += self.data[head_index + 1..].count1();
                    self.data.truncate(head_index + 1);
                    out
                }
            }
        };
        prev - self.ones
    }
}

macro_rules! set_range {
    ($this:expr, $func:ident, $i:expr, $j:expr) => {
        #[allow(clippy::range_plus_one)]
        {
            if $i < $j {
                let i = $i;
                let j = $j - 1;
                debug_assert!(i <= j);

                let (head_index, head_offset) = divmod::<usize>(i, T::BITS);
                let (last_index, last_offset) = divmod::<usize>(j, T::BITS);

                let mut out = 0;
                if head_index == last_index {
                    out += $this[head_index].$func(head_offset..last_offset + 1);
                } else {
                    out += $this[head_index].$func(head_offset..T::BITS);
                    for i in (head_index + 1)..last_index {
                        out += $this[i].$func(0..T::BITS);
                    }
                    out += $this[last_index].$func(0..last_offset + 1);
                }
                out
            } else {
                0
            }
        }
    };
}

impl<T> Assign<Range<u64>> for [T]
where
    T: FiniteBits + Assign<Range<u64>, Output = u64>,
{
    type Output = u64;

    /// Enable bits in a specified range, and returns the number of **updated** bits.
    ///
    /// # Examples
    ///
    /// ```
    /// use compacts::bits::{Count, Assign};
    /// let mut slice = [0b_11111111u8, 0b_11111111];
    /// assert_eq!(16, slice.set0(..));
    /// assert_eq!(slice, [0b_00000000u8, 0b_00000000]);
    ///
    /// assert_eq!(3, slice.set1(..=2));
    /// assert_eq!(slice, [0b_00000111u8, 0b_00000000]);
    /// assert_eq!(1, slice.set0(1..2));
    /// assert_eq!(slice, [0b_00000101u8, 0b_00000000]);
    /// assert_eq!(2, slice.set1(7..=8));
    /// assert_eq!(slice, [0b_10000101u8, 0b_00000001]);
    /// assert_eq!(4, slice.set1(7..13));
    /// assert_eq!(slice, [0b_10000101u8, 0b_00011111]);
    /// ```
    fn set1(&mut self, index: Range<u64>) -> Self::Output {
        set_range!(self, set1, index.start, index.end)
    }

    /// Disable bits in a specified range, and returns the number of **updated** bits.
    fn set0(&mut self, index: Range<u64>) -> Self::Output {
        set_range!(self, set0, index.start, index.end)
    }
}

pub struct MapIter<'a, T> {
    iter: std::slice::Iter<'a, T>,
}

impl<'a, T: FiniteBits> Iterator for MapIter<'a, T> {
    type Item = Cow<'a, T>;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|v| Cow::Borrowed(v))
    }
}

impl<'a, T: FiniteBits> IntoIterator for &'a Map<T> {
    type Item = Cow<'a, T>;
    type IntoIter = MapIter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        let iter = self.data.iter();
        MapIter { iter }
    }
}

macro_rules! implMask {
    ( $([ $($constraints:tt)*] for $Type:ty ;)+ ) => {
        $(
            impl<$($constraints)*> $Type {
                pub fn and<Rhs>(&self, rhs: Rhs) -> mask::And<&Self, Rhs> {
                    and(self, rhs)
                }
                pub fn or<Rhs>(&self, rhs: Rhs) -> mask::Or<&Self, Rhs> {
                    or(self, rhs)
                }
                pub fn xor<Rhs>(&self, rhs: Rhs) -> mask::Xor<&Self, Rhs> {
                    xor(self, rhs)
                }
                // pub fn not(self) -> Not<Self> {
                //     not(self)
                // }
            }

            // impl<$($constraints)*> std::ops::Not for $Type {
            //     type Output = Not<Self>;
            //     fn not(self) -> Self::Output {
            //         not(self)
            //     }
            // }
        )+
    }
}
implMask!(
    [T] for Map<T>;
    [K: UnsignedInt, V] for PageMap<K, V>;
);

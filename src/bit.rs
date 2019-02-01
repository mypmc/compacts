//! Module bits defines traits and types to interact with a bits container.
//!

// # References
//
// - Compact Data Structures: A Practical Approach
// - Fast, Small, Simple Rank/Select on Bitmaps
// - Space-Efficient, High-Performance Rank & Select Structures on Uncompressed Bit Sequences

#[cfg(test)]
mod tests;

// mod flip;
// mod range;

pub mod ops;
pub mod rrr;

mod block;
mod entry;
mod impls;
mod key_map;
mod vec_map;

mod mask;
mod uint;

use std::ops::{Bound, Range, RangeBounds};

use self::{
    ops::*,
    uint::{TryCast, UnsignedInt},
};

pub use self::{
    block::{Block, BlockArray},
    entry::Entry,
    mask::{and, or, xor, And, Or, Xor},
    mask::{Fold, Mask},
};

const MAX: u64 = 1 << 63;

// Panic message.
static OUT_OF_BOUNDS: &str = "index out of bounds";

/// `Map<T>`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Map<T> {
    ones: u64,
    data: Vec<T>,
}

/// `VecMap<A>` is a type alias for `Map<Block<A>>.
pub type VecMap<A> = Map<Block<A>>;

/// `KeyMap<K, V>` is a type alias for `Map<Entry<K, V>>`.
/// `KeyMap<K, V>` can be seen as a bits container that filtered out the empty `V` from `Map<V>`.
///
/// The type parameters `K` specifies the bit size of `KeyMap<K, V>`.
/// In other words, the smaller of `(1 << K::BITS) * V::BITS` and `MAX_BITS` is the bit size of `KeyMap<K, V>`.
///
/// However, there is no guaranteed that the number of bits reach that size.
/// It can fail to allocate at any point before that size is reached.
pub type KeyMap<K, V> = Map<Entry<K, V>>;

impl<T> Default for Map<T> {
    fn default() -> Self {
        Self::new_unchecked(0, Vec::new())
    }
}

impl<T> AsRef<[T]> for Map<T> {
    fn as_ref(&self) -> &[T] {
        self.data.as_slice()
    }
}

impl<T> Map<T> {
    pub fn new() -> Self {
        Self::new_unchecked(0, Vec::new())
    }

    fn new_unchecked(ones: u64, data: Vec<T>) -> Self {
        Map { ones, data }
    }

    pub fn with<U: AsRef<[T]>>(data: U) -> Map<T>
    where
        T: FiniteBits,
    {
        let slice = data.as_ref();
        let ones = slice.count1();
        let data = slice.to_vec();
        Map { ones, data }
    }

    pub fn build<U>(data: impl IntoIterator<Item = U>) -> Map<T>
    where
        Self: Assign<U>,
    {
        let mut map = Self::new();
        for x in data {
            map.set1(x);
        }
        map
    }

    /// Shrink an internal vector.
    pub fn shrink_to_fit(&mut self) {
        self.data.shrink_to_fit()
    }

    pub fn and<Rhs>(&self, rhs: Rhs) -> mask::And<&Self, Rhs> {
        and(self, rhs)
    }
    pub fn or<Rhs>(&self, rhs: Rhs) -> mask::Or<&Self, Rhs> {
        or(self, rhs)
    }
    pub fn xor<Rhs>(&self, rhs: Rhs) -> mask::Xor<&Self, Rhs> {
        xor(self, rhs)
    }
}

/// Cast U into T.
///
/// # Panics
///
/// Panics if given `u` does not fit in `T`.
#[inline]
fn cast<U, T>(u: U) -> T
where
    U: UnsignedInt + TryCast<T>,
    T: UnsignedInt,
{
    u.try_cast().expect("does not fit in T")
}

#[inline]
fn divmod<U: UnsignedInt>(i: u64, cap: u64) -> (U, u64)
where
    u64: TryCast<U>,
{
    (cast(i / cap), i % cap)
}

#[allow(clippy::range_plus_one)]
#[rustfmt::skip]
pub(crate) fn from_any_bounds<R: RangeBounds<u64>>(range: &'_ R, bits: u64) -> Range<u64> {
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

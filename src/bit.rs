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

#[cfg(feature = "roaring")]
pub mod roaring;

mod block;
mod entry;
mod map;
mod uint;

use std::{
    iter::Peekable,
    marker::PhantomData,
    ops::{Bound, Range, RangeBounds},
};

use self::{
    ops::*,
    uint::{TryCast, Uint},
};

pub use self::{
    block::{Block, BlockArray},
    entry::Entry,
};

/// Max size of the bits container.
///
/// However, there is no guarantee that the number of bits reach that size.
/// It can fail to allocate at any point before that size is reached.
pub const MAX: u64 = 1 << 63;

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
/// In other words, the smaller of `(1 << K::BITS) * V::BITS` and `bit::MAX` is the bit size of `KeyMap<K, V>`.
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

/// Cast U into T.
///
/// # Panics
///
/// Panics if given `u` does not fit in `T`.
#[inline]
fn cast<U, T>(u: U) -> T
where
    U: Uint + TryCast<T>,
    T: Uint,
{
    u.try_cast().expect("does not fit in T")
}

#[inline]
fn divmod<U: Uint>(i: u64, cap: u64) -> (U, u64)
where
    u64: TryCast<U>,
{
    (cast(i / cap), i % cap)
}

#[allow(clippy::range_plus_one)]
#[rustfmt::skip]
fn from_any_bounds<R: RangeBounds<u64>>(range: &'_ R, bits: u64) -> Range<u64> {
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

impl<T> Count for [T]
where
    T: FiniteBits,
{
    fn bits(&self) -> u64 {
        cast::<usize, u64>(self.len()) * T::BITS
    }

    fn count1(&self) -> u64 {
        self.iter().fold(0, |acc, w| acc + w.count1())
    }
}

impl<K, V> Count for [Entry<K, V>]
where
    K: Uint,
    V: FiniteBits,
{
    /// # Examples
    ///
    /// ```
    /// use compacts::bit::{Entry, ops::Count};
    /// let slice = [Entry::new(9u8, 0u64)];
    /// assert_eq!(slice.bits(), (1 << 8) * 64);
    /// ```
    fn bits(&self) -> u64 {
        Entry::<K, V>::potential_bits()
    }

    /// # Examples
    ///
    /// ```
    /// use compacts::bit::{Entry, ops::Count};
    /// let slice = [Entry::new(9u8, 0b_00001111_11110101u128)];
    /// assert_eq!(slice.bits(), (1 << 8) * 128); // 32768
    /// assert_eq!(slice.count1(), 10);
    /// assert_eq!(slice.count0(), 32758);
    /// ```
    fn count1(&self) -> u64 {
        self.iter().fold(0, |acc, e| acc + e.value.count1())
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
    /// use compacts::bit::ops::Access;
    /// let word = [0b_10101010_u8, 0b_11110000_u8];
    /// let bits = word.iterate().collect::<Vec<_>>();
    /// assert_eq!(bits, vec![1, 3, 5, 7, 12, 13, 14, 15]);
    /// ```
    fn iterate<'a>(&'a self) -> Box<dyn Iterator<Item = u64> + 'a> {
        Box::new(
            self.iter()
                .enumerate()
                .filter(|(_, t)| t.count1() > 0)
                .flat_map(|(i, t)| {
                    let offset = cast::<usize, u64>(i) * T::BITS;
                    t.iterate().map(move |j| j + offset)
                }),
        )
    }
}

impl<K, V> Access for [Entry<K, V>]
where
    K: Uint,
    V: FiniteBits + Access,
{
    /// Test bit at a given position.
    ///
    /// # Examples
    ///
    /// ```
    /// use compacts::bit::{Entry, ops::Access};
    /// let slice = [Entry::new(0usize, 1u16), Entry::new(5, 1)];
    /// assert!( slice.access(0));
    /// assert!(!slice.access(1));
    /// assert!( slice.access(80));
    /// assert!(!slice.access(81));
    /// assert!(!slice.access(96));
    /// ```
    ///
    /// We can create a masked bits by slicing entries.
    ///
    /// ```
    /// # use compacts::bit::{Entry, ops::Access};
    /// # let slice = [Entry::new(0usize, 1u16), Entry::new(5, 1)];
    /// let slice = &slice[1..]; // [Entry::new(5, 1)]
    /// assert!(!slice.access(0));
    /// assert!(!slice.access(1));
    /// assert!( slice.access(80));
    /// assert!(!slice.access(81));
    /// assert!(!slice.access(96));
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if index out of bounds.
    fn access(&self, i: u64) -> bool {
        assert!(i < self.bits(), OUT_OF_BOUNDS);
        let (i, o) = divmod(i, V::BITS);
        self.binary_search_by_key(&i, |e| e.index)
            .map(|k| self[k].value.access(o))
            .unwrap_or_default()
    }

    /// # Examples
    ///
    /// ```
    /// use compacts::bit::{Entry, ops::Access};
    /// let slice = [Entry::new(0usize, 1u16), Entry::new(5, 1)];
    /// let vec = slice.iterate().collect::<Vec<_>>();
    /// assert_eq!(vec, vec![0, 80]);
    /// ```
    ///
    fn iterate<'a>(&'a self) -> Box<dyn Iterator<Item = u64> + 'a> {
        Box::new(self.iter().flat_map(|page| {
            let offset = cast::<K, u64>(page.index) * V::BITS;
            page.value.iterate().map(move |i| i + offset)
        }))
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

impl<K, V> Rank for [Entry<K, V>]
where
    K: Uint,
    V: FiniteBits + Rank,
{
    /// Return the number of enabled bits in `[0, i)`.
    ///
    /// ```
    /// use compacts::bit::{Entry, ops::{Rank, Count}};
    /// let slice = [Entry::new(0usize, 0b_00001111_11110000u32), Entry::new(3, 0b_01100000_01100000)];
    /// assert_eq!(slice.rank1(10), 6);
    /// assert_eq!(slice.rank1(32), 8);
    /// assert_eq!(slice.rank1(103), 10);
    /// assert_eq!(slice.rank1(slice.bits()), slice.count1());
    /// ```
    ///
    /// Unlike `[T]`, slicing for `[Entry<K, V>]` mask the bits.
    ///
    /// ```
    /// # use compacts::bit::{Entry, ops::{Rank, Count}};
    /// # let slice = [Entry::new(0usize, 0b_00001111_11110000u32), Entry::new(3, 0b_01100000_01100000)];
    /// let slice = &slice[1..]; // [Entry::new(3, 0b_01100000_01100000)]
    /// assert_eq!(slice.rank1(10), 0);
    /// assert_eq!(slice.rank1(32), 0);
    /// assert_eq!(slice.rank1(103), 2);
    /// assert_eq!(slice.rank1(slice.bits()), slice.count1());
    /// ```
    fn rank1(&self, i: u64) -> u64 {
        assert!(i <= self.bits(), OUT_OF_BOUNDS);
        let (i, o) = divmod(i, V::BITS);
        let mut rank = 0;
        for entry in self {
            if entry.index < i {
                rank += entry.value.count1();
            } else if entry.index == i {
                rank += entry.value.rank1(o);
                break;
            } else if entry.index > i {
                break;
            }
        }
        rank
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

impl<K, V> Select1 for [Entry<K, V>]
where
    K: Uint,
    V: FiniteBits + Select1,
{
    fn select1(&self, mut n: u64) -> Option<u64> {
        for entry in self {
            let count = entry.value.count1();
            if n < count {
                // remain < count implies that select1 never be None.
                let select1 = entry.value.select1(n).expect("remain < count");
                return Some(cast::<K, u64>(entry.index) * V::BITS + select1);
            }
            n -= count;
        }
        None
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
                return Some(cast::<usize, u64>(k) * T::BITS + select0);
            }
            n -= count;
        }
        None
    }
}

impl<K, V> Select0 for [Entry<K, V>]
where
    K: Uint,
    V: FiniteBits + Select0,
{
    /// # Examples
    ///
    /// ```
    /// use compacts::bit::{Entry, ops::Select0};
    /// // [T]: 00000000 00000000 11111011 00000000 00000000 00000000 11111011 00000000 ...
    /// let slice = [Entry::new(2usize, 0b_11111011_u8), Entry::new(6, 0b_11111011)];
    /// assert_eq!(slice.select0(10), Some(10));
    /// assert_eq!(slice.select0(15), Some(15));
    /// assert_eq!(slice.select0(16), Some(18));
    /// assert_eq!(slice.select0(30), Some(37));
    /// assert_eq!(slice.select0(41), Some(50));
    /// assert_eq!(slice.select0(42), Some(56));
    /// ```
    fn select0(&self, mut c: u64) -> Option<u64> {
        if self.is_empty() {
            return if c < self.bits() { Some(c) } else { None };
        }

        let mut prev: Option<u64> = None; // prev index
        for entry in self {
            let index = cast::<K, u64>(entry.index);
            let value = &entry.value;

            let len = if let Some(p) = prev {
                // (p, index)
                index - (p + 1)
            } else {
                // [0, index)
                index
            };

            // None:    0..index
            // Some(p): p..index
            let count = value.count0() + V::BITS * len;
            if c >= count {
                prev = Some(index);
                c -= count;
                continue;
            }

            // c < count
            let select0 = || {
                use std::iter::{once, repeat_with};

                let iter = repeat_with(|| None)
                    .take(cast::<u64, usize>(len))
                    .chain(once(Some(value)));

                // this block is almost same with [T]
                let mut remain = c;
                for (k, v) in iter.enumerate() {
                    let skipped_bits = cast::<usize, u64>(k) * V::BITS;
                    let count0 = if let Some(v) = v { v.count0() } else { V::BITS };
                    if remain < count0 {
                        return skipped_bits
                            + if let Some(v) = v {
                                // remain < count implies that select0 never be None.
                                v.select0(remain).expect("remain < count")
                            } else {
                                remain
                            };
                    }
                    remain -= count0;
                }

                unreachable!()
            };

            let skipped_bits = prev.map_or(0, |p| (p + 1) * V::BITS);
            return Some(skipped_bits + select0());
        }

        let select = (cast::<K, u64>(self[self.len() - 1].index) + 1) * V::BITS + c;
        if select < self.bits() {
            Some(select)
        } else {
            None
        }
    }
}

impl<T, W> Read<W> for [T]
where
    T: Uint + Read<W> + TryCast<W>,
    W: Uint,
{
    fn read<R: std::ops::RangeBounds<u64>>(&self, r: R) -> W {
        let r = from_any_bounds(&r, self.bits());
        assert!(r.start < r.end);
        let i = r.start;
        let j = r.end - 1;
        assert!(j - i <= W::BITS && i < self.bits() && j < self.bits());

        let (head_index, head_offset) = divmod::<usize>(i, T::BITS);
        let (last_index, last_offset) = divmod::<usize>(j, T::BITS);

        if head_index == last_index {
            self[head_index].read(head_offset..last_offset + 1)
        } else {
            // head_index < last_index

            // returning value
            let mut out = W::ZERO;
            // how many bits do we have read?
            let mut len = 0;

            out |= self[head_index].read(head_offset..T::BITS);
            len += T::BITS - head_offset;

            for &n in &self[(head_index + 1)..last_index] {
                out |= cast::<T, W>(n).shiftl(len);
                len += T::BITS;
            }

            let last = self[last_index].read(0..last_offset + 1);
            // last need to be shifted to left by `len`
            debug_assert_eq!(
                cast::<W, u64>(last),
                cast::<W, u64>(last.shiftl(len).shiftr(len))
            );
            out | last.shiftl(len)
        }
    }
}

impl<A, W> Read<W> for [Block<A>]
where
    A: BlockArray + Read<W>,
    W: Uint,
{
    /// ```
    /// use compacts::bit::{Block, Map, ops::Read};
    /// let map = Map::<Block<[u64; 1024]>>::build(&[65535, 65536, 65537]);
    /// let slice = map.as_ref();
    /// assert_eq!(Read::<u64>::read(slice, 65535..65598), 0b_0111);
    /// assert_eq!(Read::<u64>::read(slice, 65536..65599), 0b_0011);
    /// ```
    fn read<R: std::ops::RangeBounds<u64>>(&self, r: R) -> W {
        let r = from_any_bounds(&r, self.bits());
        assert!(r.start < r.end);
        let i = r.start;
        let j = r.end - 1;
        assert!(j - i <= W::BITS && i < self.bits() && j < self.bits());

        let (head_index, head_offset) = divmod::<usize>(i, Block::<A>::BITS);
        let (last_index, last_offset) = divmod::<usize>(j, Block::<A>::BITS);

        if head_index == last_index {
            self[head_index].read(head_offset..last_offset + 1)
        } else {
            assert_eq!(head_index + 1, last_index);

            // returning value
            let mut out = W::ZERO;
            // how many bits do we have read?
            let mut len = 0;

            out |= self[head_index].read(head_offset..Block::<A>::BITS);
            len += Block::<A>::BITS - head_offset;

            let last = self[last_index].read(0..last_offset + 1);
            // last need to be shifted to left by `len`
            debug_assert_eq!(
                cast::<W, u64>(last),
                cast::<W, u64>(last.shiftl(len).shiftr(len))
            );
            out | last.shiftl(len)
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

    // fn flip(&mut self, i: u64) -> Self::Output {
    //     assert!(i < self.bits(), bit::OUT_OF_BOUNDS);
    //     let (i, o) = divmod::<usize>(i, T::BITS);
    //     self[i].flip(o)
    // }
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
    /// use compacts::bit::ops::{Count, Assign};
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

mod mask {
    pub trait Op {}

    #[derive(Debug)]
    pub struct And;
    #[derive(Debug)]
    pub struct Or;
    #[derive(Debug)]
    pub struct Xor;

    impl Op for And {}
    impl Op for Or {}
    impl Op for Xor {}
}

#[derive(Debug)]
pub struct Mask<L, R, O: mask::Op> {
    pub(crate) lhs: L,
    pub(crate) rhs: R,
    _op: PhantomData<O>,
}

pub fn and<L, R>(lhs: L, rhs: R) -> Mask<L, R, mask::And> {
    let _op = PhantomData;
    Mask { lhs, rhs, _op }
}
pub fn or<L, R>(lhs: L, rhs: R) -> Mask<L, R, mask::Or> {
    let _op = PhantomData;
    Mask { lhs, rhs, _op }
}
pub fn xor<L, R>(lhs: L, rhs: R) -> Mask<L, R, mask::Xor> {
    let _op = PhantomData;
    Mask { lhs, rhs, _op }
}

impl<L, R, O: mask::Op> Mask<L, R, O> {
    pub fn and<Rhs>(self, rhs: Rhs) -> Mask<Self, Rhs, mask::And> {
        and(self, rhs)
    }
    pub fn or<Rhs>(self, rhs: Rhs) -> Mask<Self, Rhs, mask::Or> {
        or(self, rhs)
    }
    pub fn xor<Rhs>(self, rhs: Rhs) -> Mask<Self, Rhs, mask::Xor> {
        xor(self, rhs)
    }

    // pub fn not(self) -> Not<Self> {
    //     not(self)
    // }
}

pub struct And<L: Iterator, R: Iterator, T> {
    pub(crate) lhs: Peekable<L>,
    pub(crate) rhs: Peekable<R>,
    _ty: PhantomData<T>,
}

pub struct Or<L: Iterator, R: Iterator, T> {
    pub(crate) lhs: Peekable<L>,
    pub(crate) rhs: Peekable<R>,
    _ty: PhantomData<T>,
}

pub struct Xor<L: Iterator, R: Iterator, T> {
    pub(crate) lhs: Peekable<L>,
    pub(crate) rhs: Peekable<R>,
    _ty: PhantomData<T>,
}

macro_rules! implMask {
    ($( $Iter:ident),* ) => ($(
        impl<L, R, T> IntoIterator for Mask<L, R, mask::$Iter>
        where
            L: IntoIterator<Item = T>,
            R: IntoIterator<Item = T>,
            $Iter<L::IntoIter, R::IntoIter, T>: Iterator<Item = T>,
        {
            type Item = T;
            type IntoIter = $Iter<L::IntoIter, R::IntoIter, T>;
            fn into_iter(self) -> Self::IntoIter {
                $Iter {
                    lhs: self.lhs.into_iter().peekable(),
                    rhs: self.rhs.into_iter().peekable(),
                    _ty: PhantomData,
                }
            }
        }
    )*)
}
implMask!(And, Or, Xor);

pub struct Fold<'a, T>(Option<BoxIter<'a, T>>);

type BoxIter<'a, T> = Box<dyn Iterator<Item = T> + 'a>;

impl<'a, T: 'a> Iterator for Fold<'a, T> {
    type Item = T;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.as_mut().and_then(|it| it.next())
    }
}

impl<'a, T: 'a> Fold<'a, T> {
    /// Combines all given iterators into one iterator by using `And`.
    ///
    /// # Examples
    ///
    /// ```
    /// use compacts::bit::{VecMap, Fold, ops::Access};
    /// let a = VecMap::<[u64; 1024]>::build(&[1, 2, 4, 5, 10]);
    /// let b = VecMap::<[u64; 1024]>::build(&[1, 3, 4, 8, 10]);
    /// let c = VecMap::<[u64; 1024]>::build(&[1, 2, 4, 9, 10]);
    /// let fold = Fold::and(vec![&a, &b, &c]).collect::<VecMap<[u64; 1024]>>();
    /// let bits = fold.iterate().collect::<Vec<_>>();
    /// assert_eq!(bits, vec![1, 4, 10]);
    /// ```
    pub fn and<U>(iters: impl IntoIterator<Item = U>) -> Self
    where
        U: IntoIterator<Item = T> + 'a,
        Mask<BoxIter<'a, T>, U, mask::And>: IntoIterator<Item = T>,
    {
        Self::fold(iters, and)
    }

    pub fn or<U>(iters: impl IntoIterator<Item = U>) -> Self
    where
        U: IntoIterator<Item = T> + 'a,
        Mask<BoxIter<'a, T>, U, mask::Or>: IntoIterator<Item = T>,
    {
        Self::fold(iters, or)
    }

    pub fn xor<U>(iters: impl IntoIterator<Item = U>) -> Self
    where
        U: IntoIterator<Item = T> + 'a,
        Mask<BoxIter<'a, T>, U, mask::Xor>: IntoIterator<Item = T>,
    {
        Self::fold(iters, xor)
    }

    fn fold<A, B>(iters: impl IntoIterator<Item = A>, func: impl Fn(BoxIter<'a, T>, A) -> B) -> Self
    where
        A: IntoIterator<Item = T> + 'a,
        B: IntoIterator<Item = T> + 'a,
    {
        let mut iters = iters.into_iter();
        Fold(if let Some(head) = iters.next() {
            let head = Box::new(head.into_iter());
            Some(iters.fold(head, |it, x| Box::new(func(it, x).into_iter())))
        } else {
            None
        })
    }
}

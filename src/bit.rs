//! Module bits defines traits and types to interact with a bits container.
//!
//! ## `Map<T>` and `[T]`
//!
//! ## `EntryMap<K, V>` and `[Entry<K, V>]`

// # References
//
// - Compact Data Structures: A Practical Approach
// - Fast, Small, Simple Rank/Select on Bitmaps
// - Space-Efficient, High-Performance Rank & Select Structures on Uncompressed Bit Sequences

#[macro_use]
mod macros;
#[cfg(test)]
mod tests;

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

// mod flip;
// mod range;

pub mod ops;

mod block;
mod entry;

mod mask;
mod uint;

use std::borrow::Cow;

use self::{
    block::BlockArray,
    ops::*,
    uint::{TryCast, UnsignedInt},
};

pub use self::{
    block::Block,
    entry::Entry,
    mask::{and, or, xor, Fold, Mask},
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

/// `KeyMap<K, V>` is a type alias for `Map<Entry<K, V>>.
/// `KeyMap<K, V>` can be seen as a bits container that filtered out the empty `V` from `Map<V>`.
///
/// The type parameters `K` specifies the bit size of `KeyMap<K, V>`.
/// In other words, the smaller of `(1 << K::BITS) * V::BITS` and `MAX_BITS` is the bit size of `KeyMap<K, V>`.
///
/// However, there is no guaranteed that the number of bits reach that size.
/// It can fail to allocate at any point before that size is reached.
pub type KeyMap<K, V> = Map<Entry<K, V>>;

/// `VecMap<A>` is a type alias for `Map<Block<A>>.
pub type VecMap<A> = Map<Block<A>>;

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

    pub fn with<U: AsRef<[T]>>(slice: U) -> Map<T>
    where
        T: FiniteBits,
    {
        let ones = slice.as_ref().iter().fold(0, |acc, t| acc + t.count1());
        let data = slice.as_ref().to_vec();
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
}

pub struct Chunks<'a, T: UnsignedInt> {
    iter: std::slice::Chunks<'a, T>,
}

pub struct Blocks<'a, T> {
    iter: std::slice::Iter<'a, T>,
}

pub struct Entries<'a, K: UnsignedInt, V> {
    iter: std::slice::Iter<'a, Entry<K, V>>,
}

pub struct PeekEntries<'a, K: UnsignedInt, V> {
    iter: std::iter::Peekable<std::slice::Iter<'a, Entry<K, V>>>,
}

macro_rules! implChunks {
    ($(($Uint:ty, $LEN:expr)),*) => ($(

        impl<'a> IntoIterator for &'a Map<$Uint> {
            type Item = Cow<'a, Block<[$Uint; $LEN]>>;
            type IntoIter = Chunks<'a, $Uint>;
            fn into_iter(self) -> Self::IntoIter {
                let iter = self.data.chunks($LEN);
                Chunks { iter }
            }
        }

        impl<'a> Iterator for Chunks<'a, $Uint> {
            type Item = Cow<'a, Block<[$Uint; $LEN]>>;
            fn next(&mut self) -> Option<Self::Item> {
                self.iter.next().map(|chunk| {
                    let mut block = Block::splat(0);
                    block.copy_from_slice(chunk);
                    Cow::Owned(block)
                })
            }
        }

        impl<'a, K: UnsignedInt> IntoIterator for &'a Map<Entry<K, $Uint>> {
            type Item = Entry<K, Cow<'a, Block<[$Uint; $LEN]>>>;
            type IntoIter = PeekEntries<'a, K, $Uint>;
            fn into_iter(self) -> Self::IntoIter {
                let iter = self.data.iter().peekable();
                PeekEntries { iter }
            }
        }

        impl<'a, K: UnsignedInt> Iterator for PeekEntries<'a, K, $Uint> {
            type Item = Entry<K, Cow<'a, Block<[$Uint; $LEN]>>>;
            fn next(&mut self) -> Option<Self::Item> {
                self.iter.next().map(|head| {
                    let mut arr = [0; $LEN];
                    let len = ucast::<usize, K>($LEN);

                    arr[ucast::<K, usize>(head.index % len)] = head.value;

                    // index of returning entry
                    let index = head.index / len;

                    while let Some(peek) = self.iter.peek() {
                        if peek.index / len != index {
                            break;
                        }
                        let item = self.iter.next().expect("next");
                        arr[ucast::<K, usize>(item.index % len)] = item.value;
                    }

                    return Entry::new(index, Cow::Owned(Block::from(arr)))
                })
            }
        }
    )*)
}
#[rustfmt::skip]
implChunks!(
    (u8,   8192usize),
    (u16,  4096usize),
    (u32,  2048usize),
    (u64,  1024usize),
    (u128, 512usize)
);

#[cfg(target_pointer_width = "32")]
implChunks!((usize, 2048usize));
#[cfg(target_pointer_width = "64")]
implChunks!((usize, 1024usize));

impl<'a, A: BlockArray> IntoIterator for &'a Map<A> {
    type Item = Cow<'a, Block<A>>;
    type IntoIter = Blocks<'a, A>;
    fn into_iter(self) -> Self::IntoIter {
        let iter = self.data.iter();
        Blocks { iter }
    }
}

impl<'a, A: BlockArray> IntoIterator for &'a Map<Block<A>> {
    type Item = Cow<'a, Block<A>>;
    type IntoIter = Blocks<'a, Block<A>>;
    fn into_iter(self) -> Self::IntoIter {
        let iter = self.data.iter();
        Blocks { iter }
    }
}

impl<'a, K: UnsignedInt, A: BlockArray> IntoIterator for &'a Map<Entry<K, Block<A>>> {
    type Item = Entry<K, Cow<'a, Block<A>>>;
    type IntoIter = Entries<'a, K, Block<A>>;
    fn into_iter(self) -> Self::IntoIter {
        let iter = self.data.iter();
        Entries { iter }
    }
}

impl<'a, A: BlockArray> Iterator for Blocks<'a, A> {
    type Item = Cow<'a, Block<A>>;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(array) = self.iter.next() {
            if array.count1() > 0 {
                return Some(Cow::Owned(Block::from(array)));
            }
        }
        None
    }
}

impl<'a, A: BlockArray> Iterator for Blocks<'a, Block<A>> {
    type Item = Cow<'a, Block<A>>;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(block) = self.iter.next() {
            if block.count1() > 0 {
                return Some(Cow::Borrowed(block));
            }
        }
        None
    }
}

impl<'a, K: UnsignedInt, A: BlockArray> Iterator for Entries<'a, K, Block<A>> {
    type Item = Entry<K, Cow<'a, Block<A>>>;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(page) = self.iter.next() {
            if page.value.count1() > 0 {
                let index = page.index;
                let value = Cow::Borrowed(&page.value);
                return Some(Entry::new(index, value));
            }
        }
        None
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

                // pub fn not(&self) -> Flip<&$Type> {
                //     let bits = self.bits();
                //     let data = self;
                //     Flip { bits, data }
                // }
            }
        )+
    }
}
implMask!(
    [T: FiniteBits] for Map<T>;
    [K: UnsignedInt, V: FiniteBits] for Map<Entry<K, V>>;
);

/// Cast U into T.
///
/// # Panics
///
/// Panics if given `u` does not fit in `T`.
#[inline]
fn ucast<U, T>(u: U) -> T
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
    (ucast(i / cap), i % cap)
}

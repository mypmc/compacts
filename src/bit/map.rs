use std::{
    borrow::Cow,
    cmp::Ordering,
    iter::FromIterator,
    ops::Range,
    ops::{BitAndAssign, BitOrAssign, BitXorAssign},
};

use crate::bit::{self, cast, divmod, ops::*, Block, BlockArray, Entry, KeyMap, Map, Uint};

impl<T> Map<T> {
    pub fn new() -> Self {
        Self::new_unchecked(0, Vec::new())
    }

    pub(super) fn new_unchecked(ones: u64, data: Vec<T>) -> Self {
        Map { ones, data }
    }

    pub fn build<U>(data: impl IntoIterator<Item = U>) -> Self
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

    pub fn and<Rhs>(&self, rhs: Rhs) -> bit::Mask<&Self, Rhs, bit::mask::And> {
        bit::and(self, rhs)
    }

    pub fn or<Rhs>(&self, rhs: Rhs) -> bit::Mask<&Self, Rhs, bit::mask::Or> {
        bit::or(self, rhs)
    }

    pub fn xor<Rhs>(&self, rhs: Rhs) -> bit::Mask<&Self, Rhs, bit::mask::Xor> {
        bit::xor(self, rhs)
    }
}

impl<T: Finite> Map<T> {
    pub fn with<U>(data: &U) -> Map<T>
    where
        U: ?Sized + AsRef<[T]>,
    {
        let ones = data.as_ref().count1();
        let data = data.as_ref().to_vec();
        Map { ones, data }
    }

    pub(super) fn access<U>(data: &U, i: u64) -> bool
    where
        T: Access,
        U: ?Sized + AsRef<[T]> + Count,
    {
        assert!(i < data.bits(), bit::OUT_OF_BOUNDS);
        let (i, o) = divmod::<usize>(i, T::BITS);
        data.as_ref().get(i).map_or(false, |t| t.access(o))
    }

    pub(super) fn rank1<U>(data: &U, i: u64) -> u64
    where
        T: Rank,
        U: ?Sized + AsRef<[T]> + Count,
    {
        assert!(i <= data.bits(), bit::OUT_OF_BOUNDS);
        let (i, o) = divmod(i, T::BITS);
        let slice = data.as_ref();
        let c = slice.iter().take(i).fold(0, |acc, b| acc + b.count1());
        let r = slice.get(i).map_or(0, |b| b.rank1(o));
        c + r
    }

    pub(super) fn select1<U>(data: &U, mut n: u64) -> Option<u64>
    where
        T: Select1,
        U: ?Sized + AsRef<[T]>,
    {
        for (k, v) in data.as_ref().iter().enumerate() {
            let count = v.count1();
            if n < count {
                let select1 = v.select1(n).expect("remain < count");
                return Some(cast::<usize, u64>(k) * T::BITS + select1);
            }
            n -= count;
        }
        None
    }
}

impl<T> Count for Map<T>
where
    T: Finite,
{
    /// # Examples
    ///
    /// ```
    /// use compacts::bit::{Map, ops::Count};
    /// let map = Map::with(&[0u64, 0b10101100000, 0b0000100000]);
    /// assert_eq!(1<<63, map.bits());
    /// assert_eq!(192,   map.as_ref().bits());
    /// ```
    fn bits(&self) -> u64 {
        bit::MAX
    }

    /// # Examples
    ///
    /// ```
    /// use compacts::bit::{Map, Block, ops::Count};
    /// let map = Map::<Block<[u64; 1024]>>::build(vec![0u64, 8, 13, 18, 1<<16]);
    /// assert_eq!(map.count1(), 5);
    /// assert_eq!(map.count1(), map.as_ref().count1());
    /// ```
    fn count1(&self) -> u64 {
        debug_assert!(self.ones <= self.bits());
        self.ones
    }
}

impl<K, V> Count for KeyMap<K, V>
where
    K: Uint,
    V: Finite,
{
    fn bits(&self) -> u64 {
        Entry::<K, V>::potential_bits()
    }
    fn count1(&self) -> u64 {
        self.ones
    }
}

impl<T> Access for Map<T>
where
    T: Finite + Access,
{
    /// Test bit at a given position.
    ///
    /// # Examples
    ///
    /// ```
    /// use compacts::bit::{Map, ops::Access};
    /// let map = Map::with(&[0b_00000101u64, 0b01100011]);
    /// assert!( map.access(0));
    /// assert!(!map.access(1));
    /// assert!( map.access(2));
    /// assert!(!map.access(16));
    /// ```
    ///
    /// The length of slice must be greater than `i % T::BITS`.
    ///
    /// ```
    /// # use compacts::bit::{Map, ops::Access};
    /// # let map = Map::with(&[0b_00000101u64, 0b01100011]);
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
    /// # use compacts::bit::{Map, ops::Access};
    /// # let map = Map::with(&[0b_00000101u64, 0b01100011]);
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
    /// use compacts::bit::ops::Access;
    /// let word = [0b_10101010_u8, 0b_11110000_u8];
    /// let bits = word.iterate().collect::<Vec<_>>();
    /// assert_eq!(bits, vec![1, 3, 5, 7, 12, 13, 14, 15]);
    /// ```
    fn iterate<'a>(&'a self) -> Box<dyn Iterator<Item = u64> + 'a> {
        Box::new(self.data.iter().enumerate().flat_map(|(i, t)| {
            let offset = cast::<usize, u64>(i) * T::BITS;
            t.iterate().map(move |j| j + offset)
        }))
    }
}

impl<K, V> Access for KeyMap<K, V>
where
    K: Uint,
    V: Finite + Access,
{
    fn access(&self, i: u64) -> bool {
        self.data.access(i)
    }
    fn iterate<'a>(&'a self) -> Box<dyn Iterator<Item = u64> + 'a> {
        self.data.iterate()
    }
}

impl<T> Rank for Map<T>
where
    T: Finite + Rank,
{
    /// Returns the number of enabled bits in `[0, i)`.
    ///
    /// The length of slice must be greater than `i % T::BITS`.
    ///
    /// ```
    /// use compacts::bit::ops::{Count, Rank};
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

impl<K, V> Rank for KeyMap<K, V>
where
    K: Uint,
    V: Finite + Rank,
{
    fn rank1(&self, i: u64) -> u64 {
        let bits = self.bits();
        assert!(i <= bits, bit::OUT_OF_BOUNDS);
        if i == bits {
            self.ones
        } else {
            self.data.rank1(i)
        }
    }
}

impl<T> Select1 for Map<T>
where
    T: Finite + Select1,
{
    /// # Examples
    ///
    /// ```
    /// use compacts::bit::{Map, ops::Select1};
    /// let map = Map::with(&[0b_00000000_u8, 0b_01000000, 0b_00001001]);
    /// assert_eq!(map.select1(0), Some(14));
    /// assert_eq!(map.select1(1), Some(16));
    /// assert_eq!(map.select1(2), Some(19));
    /// assert_eq!(map.select1(3), None);
    /// ```
    ///
    /// ```
    /// # use compacts::bit::{Map, ops::Select1};
    /// # let map = Map::with(&[0b_00000000_u8, 0b_01000000, 0b_00001001]);
    /// assert_eq!(map.as_ref().select1(0), Some(14));
    /// assert_eq!(map.as_ref().select1(1), Some(16));
    /// assert_eq!(map.as_ref().select1(2), Some(19));
    /// assert_eq!(map.as_ref().select1(3), None);
    /// ```
    fn select1(&self, n: u64) -> Option<u64> {
        Map::select1(self, n)
    }
}

impl<K, V> Select1 for KeyMap<K, V>
where
    K: Uint,
    V: Finite + Select1,
{
    fn select1(&self, n: u64) -> Option<u64> {
        if n < self.count1() {
            self.data.select1(n)
        } else {
            None
        }
    }
}

impl<T> Select0 for Map<T>
where
    T: Finite + Select0,
{
    /// # Examples
    ///
    /// ```
    /// use compacts::bit::{Map, ops::Select0};
    /// let map = Map::with(&[0b_11110111_u8, 0b_11111110, 0b_10010011]);
    /// assert_eq!(map.select0(0), Some(3));
    /// assert_eq!(map.select0(1), Some(8));
    /// assert_eq!(map.select0(2), Some(18));
    /// assert_eq!(map.select0(6), Some(24));
    /// ```
    /// ```
    /// # use compacts::bit::{Map, ops::Select0};
    /// # let map = Map::with(&[0b_11110111_u8, 0b_11111110, 0b_10010011]);
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
                return Some(cast::<usize, u64>(k) * T::BITS + select0);
            }
            n -= count;
        }
        let select = cast::<usize, u64>(self.data.len()) * T::BITS + n;
        if select < self.bits() {
            Some(select)
        } else {
            None
        }
    }
}

impl<K, V> Select0 for KeyMap<K, V>
where
    K: Uint,
    V: Finite + Select0,
{
    fn select0(&self, n: u64) -> Option<u64> {
        if n < self.count0() {
            self.data.select0(n)
        } else {
            None
        }
    }
}

impl<T> Assign<u64> for Map<T>
where
    T: Finite + Access + Assign<u64>,
{
    type Output = ();

    /// Enable bit at a given position.
    ///
    /// # Examples
    ///
    /// ```
    /// use compacts::bit::{Map, Block, ops::{Access, Assign}};
    /// let mut map = Map::with(&[0u64, 0b10101100000, 0b0000100000]);
    /// map.set1(0);
    /// map.set1(2);
    /// assert!( map.access(0));
    /// assert!(!map.access(1));
    /// assert!( map.access(2));
    ///
    /// let map = Map::<Block<[u64; 1024]>>::build(vec![0u64, 2]);
    /// assert!( map.access(0));
    /// assert!(!map.access(1));
    /// assert!( map.access(2));
    /// ```
    ///
    /// The length of slice must be greater than `i % T::BITS`.
    ///
    fn set1(&mut self, i: u64) -> Self::Output {
        assert!(i < self.bits(), bit::OUT_OF_BOUNDS);
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
    /// use compacts::bit::ops::{Access, Assign};
    /// let mut slice = [0u64, 0b10101100001, 0b0000100000];
    /// assert!( slice.access(64));
    /// slice.set0(64);
    /// assert!(!slice.access(64));
    /// ```
    ///
    /// The length of slice must be greater than `i % T::BITS`.
    ///
    fn set0(&mut self, i: u64) -> Self::Output {
        assert!(i < self.bits(), bit::OUT_OF_BOUNDS);
        let (i, o) = divmod::<usize>(i, T::BITS);
        if i < self.data.len() && self.data[i].access(o) {
            self.ones -= 1;
            self.data[i].set0(o);
        }
    }
}

impl<K, V> Assign<u64> for KeyMap<K, V>
where
    K: Uint,
    V: Finite + Access + Assign<u64>,
{
    type Output = ();
    fn set1(&mut self, i: u64) -> Self::Output {
        assert!(i < self.bits(), bit::OUT_OF_BOUNDS);
        let (index, offset) = bit::divmod(i, V::BITS);
        match Entry::find(&*self.data, &index) {
            Ok(j) => {
                if !self.data[j].value.access(offset) {
                    self.ones += 1;
                    self.data[j].value.set1(offset);
                }
            }
            Err(j) => {
                self.ones += 1;
                let mut value = V::empty();
                value.set1(offset);
                let entry = Entry::new(index, value);
                self.data.insert(j, entry);
            }
        }
    }

    fn set0(&mut self, i: u64) -> Self::Output {
        assert!(i < self.bits(), bit::OUT_OF_BOUNDS);
        let (index, offset) = bit::divmod(i, V::BITS);
        if let Ok(k) = Entry::find(&*self.data, &index) {
            if self.data[k].value.access(offset) {
                self.ones -= 1;
                self.data[k].value.set0(offset);
                if self.data[k].value.count1() == 0 {
                    self.data.remove(k);
                }
            }
        }
    }
}

impl<T> Assign<Range<u64>> for Map<T>
where
    T: Finite + Assign<Range<u64>, Output = u64>,
{
    type Output = u64;

    /// # Examples
    ///
    /// ```
    /// use compacts::bit::{Map, ops::Assign};
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
                if head_index >= self.data.len() {
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

impl<'a, V, U> FromIterator<Cow<'a, V>> for Map<U>
where
    V: Clone + Count + 'a,
    U: From<V>,
{
    fn from_iter<I>(iterable: I) -> Self
    where
        I: IntoIterator<Item = Cow<'a, V>>,
    {
        let mut ones = 0;
        let mut bits = Vec::with_capacity(1 << 10);

        iterable.into_iter().for_each(|cow| {
            let count = cow.as_ref().count1();
            if count == 0 {
                return;
            }
            ones += count;
            let value = cow.into_owned().into();
            bits.push(value);
        });

        bits.shrink_to_fit();
        Self::new_unchecked(ones, bits)
    }
}

impl<'a, K, V, U> FromIterator<Entry<K, Cow<'a, V>>> for KeyMap<K, U>
where
    K: Uint,
    V: Clone + Count + 'a,
    U: From<V>,
{
    fn from_iter<I>(iterable: I) -> Self
    where
        I: IntoIterator<Item = Entry<K, Cow<'a, V>>>,
    {
        let mut ones = 0;
        let mut bits = Vec::with_capacity(1 << 10);

        iterable.into_iter().for_each(|entry| {
            let count = entry.value.as_ref().count1();
            if count == 0 {
                return;
            }
            ones += count;
            let value = entry.value.into_owned().into();
            bits.push(Entry::new(entry.index, value));
        });

        bits.shrink_to_fit();
        KeyMap::new_unchecked(ones, bits)
    }
}

pub struct Chunks<'a, T: Uint> {
    iter: std::slice::Chunks<'a, T>,
}

pub struct Blocks<'a, T> {
    iter: std::slice::Iter<'a, T>,
}

pub struct Entries<'a, K: Uint, V> {
    iter: std::slice::Iter<'a, Entry<K, V>>,
}

pub struct PeekEntries<'a, K: Uint, V> {
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

        impl<'a, K: Uint> IntoIterator for &'a KeyMap<K, $Uint> {
            type Item = Entry<K, Cow<'a, Block<[$Uint; $LEN]>>>;
            type IntoIter = PeekEntries<'a, K, $Uint>;
            fn into_iter(self) -> Self::IntoIter {
                let iter = self.data.iter().peekable();
                PeekEntries { iter }
            }
        }

        impl<'a, K: Uint> Iterator for PeekEntries<'a, K, $Uint> {
            type Item = Entry<K, Cow<'a, Block<[$Uint; $LEN]>>>;
            fn next(&mut self) -> Option<Self::Item> {
                self.iter.next().map(|head| {
                    let mut arr = [0; $LEN];
                    let len = cast::<usize, K>($LEN);

                    arr[cast::<K, usize>(head.index % len)] = head.value;

                    // index of returning entry
                    let index = head.index / len;

                    while let Some(peek) = self.iter.peek() {
                        if peek.index / len != index {
                            break;
                        }
                        let item = self.iter.next().expect("next");
                        arr[cast::<K, usize>(item.index % len)] = item.value;
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

impl<'a, A> IntoIterator for &'a Map<A>
where
    A: BlockArray,
{
    type Item = Cow<'a, Block<A>>;
    type IntoIter = Blocks<'a, A>;
    fn into_iter(self) -> Self::IntoIter {
        let iter = self.data.iter();
        Blocks { iter }
    }
}

impl<'a, A> IntoIterator for &'a Map<Block<A>>
where
    A: BlockArray,
{
    type Item = Cow<'a, Block<A>>;
    type IntoIter = Blocks<'a, Block<A>>;
    fn into_iter(self) -> Self::IntoIter {
        let iter = self.data.iter();
        Blocks { iter }
    }
}

impl<'a, K, A> IntoIterator for &'a KeyMap<K, Block<A>>
where
    K: Uint,
    A: BlockArray,
{
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

impl<'a, K, A> Iterator for Entries<'a, K, Block<A>>
where
    K: bit::Uint,
    A: BlockArray,
{
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

impl<'a, L, R, A> Iterator for bit::And<L, R, Cow<'a, Block<A>>>
where
    L: Iterator<Item = Cow<'a, Block<A>>>,
    R: Iterator<Item = Cow<'a, Block<A>>>,
    A: BlockArray,
{
    type Item = Cow<'a, Block<A>>;
    fn next(&mut self) -> Option<Self::Item> {
        let lhs = &mut self.lhs;
        let rhs = &mut self.rhs;
        lhs.next().and_then(|mut x| {
            rhs.next().map(|y| {
                x.to_mut().bitand_assign(y.as_ref());
                x
            })
        })
    }
}

impl<'a, L, R, A> Iterator for bit::Or<L, R, Cow<'a, Block<A>>>
where
    L: Iterator<Item = Cow<'a, Block<A>>>,
    R: Iterator<Item = Cow<'a, Block<A>>>,
    A: BlockArray,
{
    type Item = Cow<'a, Block<A>>;
    fn next(&mut self) -> Option<Self::Item> {
        match (self.lhs.next(), self.rhs.next()) {
            (Some(mut x), Some(y)) => {
                x.to_mut().bitor_assign(y.as_ref());
                Some(x)
            }
            (Some(x), None) => Some(x),
            (None, Some(y)) => Some(y),
            (None, None) => None,
        }
    }
}

impl<'a, L, R, A> Iterator for bit::Xor<L, R, Cow<'a, Block<A>>>
where
    L: Iterator<Item = Cow<'a, Block<A>>>,
    R: Iterator<Item = Cow<'a, Block<A>>>,
    A: BlockArray,
{
    type Item = Cow<'a, Block<A>>;
    fn next(&mut self) -> Option<Self::Item> {
        match (self.lhs.next(), self.rhs.next()) {
            (Some(mut x), Some(y)) => {
                x.to_mut().bitxor_assign(y.as_ref());
                Some(x)
            }
            (Some(x), None) => Some(x),
            (None, Some(y)) => Some(y),
            (None, None) => None,
        }
    }
}

impl<'a, L, R, K, A> Iterator for bit::And<L, R, Entry<K, Cow<'a, Block<A>>>>
where
    L: Iterator<Item = Entry<K, Cow<'a, Block<A>>>>,
    R: Iterator<Item = Entry<K, Cow<'a, Block<A>>>>,
    K: Uint,
    A: BlockArray,
{
    type Item = Entry<K, Cow<'a, Block<A>>>;
    fn next(&mut self) -> Option<Self::Item> {
        let lhs = &mut self.lhs;
        let rhs = &mut self.rhs;

        loop {
            match lhs
                .peek()
                .and_then(|x| rhs.peek().map(|y| x.index.cmp(&y.index)))
            {
                Some(Ordering::Equal) => {
                    let mut lhs = lhs.next().expect("peek");
                    let rhs = rhs.next().expect("peek");
                    lhs.value.to_mut().bitand_assign(rhs.value.as_ref());
                    break Some(lhs);
                }
                Some(Ordering::Less) => {
                    lhs.next();
                }
                Some(Ordering::Greater) => {
                    rhs.next();
                }
                None => break None,
            }
        }
    }
}

impl<'a, L, R, K, A> Iterator for bit::Or<L, R, Entry<K, Cow<'a, Block<A>>>>
where
    L: Iterator<Item = Entry<K, Cow<'a, Block<A>>>>,
    R: Iterator<Item = Entry<K, Cow<'a, Block<A>>>>,
    K: Uint,
    A: BlockArray,
{
    type Item = Entry<K, Cow<'a, Block<A>>>;
    fn next(&mut self) -> Option<Self::Item> {
        let lhs = &mut self.lhs;
        let rhs = &mut self.rhs;

        match (lhs.peek(), rhs.peek()) {
            (Some(l), Some(r)) => match l.index.cmp(&r.index) {
                Ordering::Less => lhs.next(),
                Ordering::Equal => {
                    let mut lhs = lhs.next().expect("peek");
                    let rhs = rhs.next().expect("peek");
                    lhs.value.to_mut().bitor_assign(rhs.value.as_ref());
                    Some(lhs)
                }
                Ordering::Greater => rhs.next(),
            },
            (Some(_), None) => lhs.next(),
            (None, Some(_)) => rhs.next(),
            (None, None) => None,
        }
    }
}

impl<'a, L, R, K, A> Iterator for bit::Xor<L, R, Entry<K, Cow<'a, Block<A>>>>
where
    L: Iterator<Item = Entry<K, Cow<'a, Block<A>>>>,
    R: Iterator<Item = Entry<K, Cow<'a, Block<A>>>>,
    K: Uint,
    A: BlockArray,
{
    type Item = bit::Entry<K, Cow<'a, Block<A>>>;
    fn next(&mut self) -> Option<Self::Item> {
        let lhs = &mut self.lhs;
        let rhs = &mut self.rhs;

        match (lhs.peek(), rhs.peek()) {
            (Some(l), Some(r)) => match l.index.cmp(&r.index) {
                Ordering::Less => lhs.next(),
                Ordering::Equal => {
                    let mut lhs = lhs.next().expect("peek");
                    let rhs = rhs.next().expect("peek");
                    lhs.value.to_mut().bitxor_assign(rhs.value.as_ref());
                    Some(lhs)
                }
                Ordering::Greater => rhs.next(),
            },
            (Some(_), None) => lhs.next(),
            (None, Some(_)) => rhs.next(),
            (None, None) => None,
        }
    }
}

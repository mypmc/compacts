use crate::bits::*;

use std::{borrow::Cow, ops::Range};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Map<T> {
    pub(crate) ones: u64,
    pub(crate) data: Vec<T>,
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

    pub fn new_unchecked(ones: u64, data: Vec<T>) -> Self {
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

    // pub fn into_vec(self) -> Vec<T> {
    //     self.data
    // }

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
    /// use compacts::bits::{Map, ops::Count};
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
    /// use compacts::bits::{Map, Block, ops::Count};
    /// let map = Map::<Block<[u64; 1024]>>::build(vec![0u64, 8, 13, 18, 1<<16]);
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
    /// use compacts::bits::{Map, ops::Access};
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
    /// # use compacts::bits::{Map, ops::Access};
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
    /// # use compacts::bits::{Map, ops::Access};
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
    /// use compacts::bits::ops::Access;
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
    /// use compacts::bits::ops::Access;
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
                    let offset = ucast::<usize, u64>(i) * T::BITS;
                    t.iterate().map(move |j| j + offset)
                }),
        )
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
    /// use compacts::bits::ops::{Count, Rank};
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
    /// use compacts::bits::{Map, ops::Select1};
    /// let map = Map::with([0b_00000000_u8, 0b_01000000, 0b_00001001]);
    /// assert_eq!(map.select1(0), Some(14));
    /// assert_eq!(map.select1(1), Some(16));
    /// assert_eq!(map.select1(2), Some(19));
    /// assert_eq!(map.select1(3), None);
    /// ```
    ///
    /// ```
    /// # use compacts::bits::{Map, ops::Select1};
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
    /// use compacts::bits::{Map, ops::Select0};
    /// let map = Map::with([0b_11110111_u8, 0b_11111110, 0b_10010011]);
    /// assert_eq!(map.select0(0), Some(3));
    /// assert_eq!(map.select0(1), Some(8));
    /// assert_eq!(map.select0(2), Some(18));
    /// assert_eq!(map.select0(6), Some(24));
    /// ```
    /// ```
    /// # use compacts::bits::{Map, ops::Select0};
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
    /// use compacts::bits::{Map, Block, ops::{Access, Assign}};
    /// let mut map = Map::with([0u64, 0b10101100000, 0b0000100000]);
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
    /// use compacts::bits::ops::{Access, Assign};
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

    // fn flip(&mut self, i: u64) -> Self::Output {
    //     assert!(i < self.bits(), OUT_OF_BOUNDS);
    //     let (i, o) = divmod::<usize>(i, T::BITS);
    //     if i < self.data.len() {
    //         if self.data[i].access(o) {
    //             self.ones -= 1;
    //             self.data[i].set0(o)
    //         } else {
    //             self.ones += 1;
    //             self.data[i].set1(o)
    //         }
    //     } else {
    //         self.ones += 1;
    //         self.data.resize(i + 1, T::empty());
    //         self.data[i].set1(o)
    //     }
    // }
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
    //     assert!(i < self.bits(), OUT_OF_BOUNDS);
    //     let (i, o) = divmod::<usize>(i, T::BITS);
    //     self[i].flip(o)
    // }
}

impl<T> Assign<Range<u64>> for Map<T>
where
    T: FiniteBits + Assign<Range<u64>, Output = u64>,
{
    type Output = u64;

    /// # Examples
    ///
    /// ```
    /// use compacts::bits::{Map, ops::Assign};
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
    /// use compacts::bits::ops::{Count, Assign};
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

impl<'a, V, U> std::iter::FromIterator<Cow<'a, V>> for Map<U>
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
        Map::new_unchecked(ones, bits)
    }
}

impl<'a, K, V, U> std::iter::FromIterator<Entry<K, Cow<'a, V>>> for Map<Entry<K, U>>
where
    K: UnsignedInt,
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
        EntryMap::new_unchecked(ones, bits)
    }
}

pub struct Chunks<'a, T: UnsignedInt> {
    iter: std::slice::Chunks<'a, T>,
}

pub struct Blocks<'a, T> {
    iter: std::slice::Iter<'a, T>,
}

pub struct Entrys<'a, K: UnsignedInt, V> {
    iter: std::slice::Iter<'a, Entry<K, V>>,
}

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
    type IntoIter = Entrys<'a, K, Block<A>>;
    fn into_iter(self) -> Self::IntoIter {
        let iter = self.data.iter();
        Entrys { iter }
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

impl<'a, K: UnsignedInt, A: BlockArray> Iterator for Entrys<'a, K, Block<A>> {
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

macro_rules! implIntoIteratorForBoxMap {
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

        // impl<'a, K: UnsignedInt> IntoIterator for &'a Map<Entry<K, $Uint>> {
        //     type Item = Entry<K, Cow<'a, [$Uint]>>;
        //     type IntoIter = Entrys<'a, K, $Uint>;
        //     fn into_iter(self) -> Self::IntoIter {
        //         let iter = self.data.iter();
        //         Entrys { iter }
        //     }
        // }
        // impl<'a, K: UnsignedInt> Iterator for Entrys<'a, K, $Uint> {
        //     type Item = Entry<K, Cow<'a, [$Uint]>>;
        //     fn next(&mut self) -> Option<Self::Item> {
        //         unimplemented!()
        //     }
        // }
    )*)
}

#[rustfmt::skip]
implIntoIteratorForBoxMap!(
    (u8,   8192usize),
    (u16,  4096usize),
    (u32,  2048usize),
    (u64,  1024usize),
    (u128, 512usize)
);

#[cfg(target_pointer_width = "32")]
implIntoIteratorForBoxMap!((usize, 2048usize));
#[cfg(target_pointer_width = "64")]
implIntoIteratorForBoxMap!((usize, 1024usize));

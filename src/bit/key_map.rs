use std::{borrow::Cow, iter::FromIterator};

use crate::bit::{self, cast, ops::*, Entry, KeyMap};

impl<K: bit::UnsignedInt, V> Count for KeyMap<K, V>
where
    V: FiniteBits,
{
    fn bits(&self) -> u64 {
        Entry::<K, V>::potential_bits()
    }
    fn count1(&self) -> u64 {
        self.ones
    }
}

impl<K: bit::UnsignedInt, V> Count for [Entry<K, V>]
where
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

impl<K: bit::UnsignedInt, V> Access for bit::KeyMap<K, V>
where
    V: FiniteBits + Access,
{
    fn access(&self, i: u64) -> bool {
        self.data.access(i)
    }
    fn iterate<'a>(&'a self) -> Box<dyn Iterator<Item = u64> + 'a> {
        self.data.iterate()
    }
}

impl<K: bit::UnsignedInt, V> Access for [Entry<K, V>]
where
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
        assert!(i < self.bits(), bit::OUT_OF_BOUNDS);
        let (i, o) = bit::divmod(i, V::BITS);
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

impl<K: bit::UnsignedInt, V> Assign<u64> for bit::KeyMap<K, V>
where
    V: FiniteBits + Access + Assign<u64>,
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

impl<K: bit::UnsignedInt, V> Rank for bit::KeyMap<K, V>
where
    V: FiniteBits + Rank,
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

impl<K: bit::UnsignedInt, V> Rank for [Entry<K, V>]
where
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
        assert!(i <= self.bits(), bit::OUT_OF_BOUNDS);
        let (i, o) = bit::divmod(i, V::BITS);
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

impl<K: bit::UnsignedInt, V> Select1 for bit::KeyMap<K, V>
where
    V: FiniteBits + Select1,
{
    fn select1(&self, n: u64) -> Option<u64> {
        if n < self.count1() {
            self.data.select1(n)
        } else {
            None
        }
    }
}

impl<K: bit::UnsignedInt, V> Select1 for [Entry<K, V>]
where
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

impl<K: bit::UnsignedInt, V> Select0 for bit::KeyMap<K, V>
where
    V: FiniteBits + Select0,
{
    fn select0(&self, n: u64) -> Option<u64> {
        if n < self.count0() {
            self.data.select0(n)
        } else {
            None
        }
    }
}

impl<K: bit::UnsignedInt, V> Select0 for [Entry<K, V>]
where
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

impl<'a, K, V, U> FromIterator<bit::Entry<K, Cow<'a, V>>> for bit::KeyMap<K, U>
where
    K: bit::UnsignedInt,
    V: Clone + Count + 'a,
    U: From<V>,
{
    fn from_iter<I>(iterable: I) -> Self
    where
        I: IntoIterator<Item = bit::Entry<K, Cow<'a, V>>>,
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
            bits.push(bit::Entry::new(entry.index, value));
        });

        bits.shrink_to_fit();
        bit::KeyMap::new_unchecked(ones, bits)
    }
}

pub struct Entries<'a, K: bit::UnsignedInt, V> {
    iter: std::slice::Iter<'a, Entry<K, V>>,
}

pub struct PeekEntries<'a, K: bit::UnsignedInt, V> {
    iter: std::iter::Peekable<std::slice::Iter<'a, Entry<K, V>>>,
}

macro_rules! implChunks {
    ($(($Uint:ty, $LEN:expr)),*) => ($(
        impl<'a, K: bit::UnsignedInt> IntoIterator for &'a bit::KeyMap<K, $Uint> {
            type Item = Entry<K, Cow<'a, bit::Block<[$Uint; $LEN]>>>;
            type IntoIter = PeekEntries<'a, K, $Uint>;
            fn into_iter(self) -> Self::IntoIter {
                let iter = self.data.iter().peekable();
                PeekEntries { iter }
            }
        }

        impl<'a, K: bit::UnsignedInt> Iterator for PeekEntries<'a, K, $Uint> {
            type Item = Entry<K, Cow<'a, bit::Block<[$Uint; $LEN]>>>;
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

                    return Entry::new(index, Cow::Owned(bit::Block::from(arr)))
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

impl<'a, K, A> IntoIterator for &'a bit::KeyMap<K, bit::Block<A>>
where
    K: bit::UnsignedInt,
    A: bit::BlockArray,
{
    type Item = Entry<K, Cow<'a, bit::Block<A>>>;
    type IntoIter = Entries<'a, K, bit::Block<A>>;
    fn into_iter(self) -> Self::IntoIter {
        let iter = self.data.iter();
        Entries { iter }
    }
}

impl<'a, K, A> Iterator for Entries<'a, K, bit::Block<A>>
where
    K: bit::UnsignedInt,
    A: bit::BlockArray,
{
    type Item = Entry<K, Cow<'a, bit::Block<A>>>;
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

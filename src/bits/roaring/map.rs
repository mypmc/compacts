use std::{
    borrow::Cow,
    io,
    iter::{FromIterator, Zip},
    slice::Iter as SliceIter,
};

use crate::{
    // bits::{
    //     index::{BitIndex, BitIndexMut},
    //     mask::*,
    // },
    num::*,
    ops::*,
    roaring::{repr::REPR_POS1_LEN, BitMap, Block, Bytes, Header, Repr},
};

use byteorder::{ReadBytesExt, WriteBytesExt, LE};

impl Default for BitMap {
    fn default() -> Self {
        Self::new_unchecked(Vec::new(), Vec::new())
    }
}

impl BitMap {
    pub fn new() -> Self {
        Default::default()
    }

    pub(super) fn new_unchecked(keys: Vec<u16>, data: Vec<Block>) -> Self {
        BitMap { keys, data }
    }

    pub fn of<'a>(data: impl IntoIterator<Item = &'a u64>) -> Self {
        let mut map = Self::new();
        for &i in data {
            map.put1(i);
        }
        map
    }

    pub fn keys(&self) -> Keys<'_, u16> {
        Keys {
            iter: self.keys.iter(),
        }
    }

    pub fn values(&self) -> Values<'_> {
        Values {
            iter: self.data.iter(),
        }
    }

    pub fn steps(&self) -> Steps<'_, u16> {
        Steps {
            zipped: self.keys().zip(self.values()),
        }
    }
}

enum Entry<'a> {
    Occupy {
        idx: usize,
        map: &'a mut BitMap,
    },
    Vacant {
        idx: usize,
        key: u16,
        map: &'a mut BitMap,
    },
}

impl BitMap {
    fn block(&self, key: u16) -> Option<&'_ Block> {
        self.keys.binary_search(&key).map(|i| &self.data[i]).ok()
    }

    // fn slice(&self, key: u16) -> Option<Slice<'_>> {
    //     self.block(key).map(|b| b.into())
    // }

    // pub fn blocks(&self) -> impl Iterator<Item = Step<u16, &'_ Block>> {
    //     let keys = self.keys.iter();
    //     let vals = self.data.iter();
    //     keys.zip(vals).map(|(&index, value)| Step { index, value })
    // }

    fn into_blocks(self) -> impl Iterator<Item = (u16, Block)> {
        let keys = self.keys.into_iter();
        let vals = self.data.into_iter();
        keys.zip(vals).map(|(index, value)| (index, value))
    }

    fn entry<'a>(&'a mut self, key: &u16) -> Entry<'a> {
        match self.keys.binary_search(key) {
            Ok(idx) => Entry::Occupy { idx, map: self },
            Err(idx) => Entry::Vacant {
                idx,
                key: *key,
                map: self,
            },
        }
    }

    pub fn into_vec(self) -> Vec<Block> {
        assert_eq!(self.keys.len(), self.data.len());
        let mut blocks = if let Some(&last_key) = self.keys.last() {
            let last_index = try_cast::<u16, usize>(last_key);
            Vec::with_capacity(last_index)
        } else {
            Vec::new()
        };

        for (index, value) in self.into_blocks() {
            let i = try_cast::<u16, usize>(index);
            if blocks.len() < i + 1 {
                blocks.resize(i + 1, Block::default());
            }
            blocks.insert(i, value);
        }
        blocks
    }

    pub fn optimize(&mut self) {
        self.shrink_to_fit();
        for Block(repr) in &mut self.data {
            repr.optimize();
        }
    }

    /// Shrink an internal vector.
    pub fn shrink_to_fit(&mut self) {
        let len = {
            assert_eq!(self.keys.len(), self.data.len());
            self.data.len()
        };

        let mut del = 0;
        for i in 0..len {
            if Bits::make(&self.data).count1() == 0 {
                del += 1;
            } else if del > 0 {
                self.keys.swap(i - del, i);
                self.data.swap(i - del, i);
            }
        }
        self.keys.truncate(len - del);
        self.data.truncate(len - del);
    }
}

impl<'a> Entry<'a> {
    fn or_empty(self) -> &'a mut Block {
        match self {
            Entry::Occupy { idx, map } => &mut map.data[idx],
            Entry::Vacant { idx, key, map } => {
                map.keys.insert(idx, key);
                map.data.insert(idx, Block::default());
                &mut map.data[idx]
            }
        }
    }

    fn put1(self, i: u64) -> bool {
        self.or_empty().put1(i)
    }

    fn put0(self, i: u64) -> bool {
        match self {
            Entry::Occupy { idx, map } => {
                let result = map.data[idx].put0(i);
                if map.data[idx].count1() == 0 {
                    let _key = map.keys.remove(idx);
                    let _val = map.data.remove(idx);
                }
                result
            }
            Entry::Vacant { .. } => false,
        }
    }

    fn putn<W: Word>(self, i: u64, len: u64, num: W) {
        self.or_empty().putn(i, len, num)
    }
}

impl BitMap {
    /// Size in bits.
    #[inline(always)]
    pub fn len(&self) -> u64 {
        Bits::len(self)
    }

    #[inline(always)]
    pub fn count1(&self) -> u64 {
        Bits::count1(self)
    }

    #[inline(always)]
    pub fn count0(&self) -> u64 {
        Bits::count0(self)
    }

    #[inline(always)]
    pub fn all(&self) -> bool {
        Bits::all(self)
    }

    #[inline(always)]
    pub fn any(&self) -> bool {
        Bits::any(self)
    }

    /// Test bit at a given position.
    ///
    /// # Panics
    ///
    /// Panics if `i >= self.len()`.
    #[inline(always)]
    pub fn get(&self, i: u64) -> bool {
        Bits::get(self, i)
    }

    #[inline(always)]
    pub fn put(&mut self, i: u64, v: bool) -> bool {
        BitsMut::put(self, i, v)
    }

    /// Enables bit at a given position.
    #[inline(always)]
    pub fn put1(&mut self, i: u64) -> bool {
        BitsMut::put1(self, i)
    }

    /// Disables bit at a given position.
    #[inline(always)]
    pub fn put0(&mut self, i: u64) -> bool {
        BitsMut::put0(self, i)
    }

    #[inline(always)]
    pub fn set1<I>(&mut self, index: I) -> u64
    where
        I: BitIndexMut<BitMap>,
    {
        index.set1(self)
    }

    #[inline(always)]
    pub fn set0<I>(&mut self, index: I) -> u64
    where
        I: BitIndexMut<BitMap>,
    {
        index.set0(self)
    }

    #[inline(always)]
    pub fn flip<I>(&mut self, index: I)
    where
        I: BitIndexMut<BitMap>,
    {
        index.flip(self)
    }

    /// Returns the number of enabled bits within the given index.
    ///
    /// ```
    /// # let vec = compacts::BitMap::<u8>::of(&[]);
    /// # assert_eq!(vec.rank0(0), 0);
    /// # assert_eq!(vec.rank1(0), 0);
    ///
    /// let vec = compacts::BitMap::<u8>::of(&[10, 20, 80, 65536, 65579]);
    /// assert_eq!(vec.rank1(0),  0);
    /// assert_eq!(vec.rank1(80), 2);
    ///
    /// # assert_eq!(vec.rank1(  ..  ), vec.count1());
    /// # assert_eq!(vec.rank1( 0.. 0), 0);
    /// # assert_eq!(vec.rank1(10..10), 0);
    /// # assert_eq!(vec.rank1(20..20), 0);
    ///
    /// assert_eq!(vec.rank1(     ..   80), 2);
    /// assert_eq!(vec.rank1(   10..   80), 2);
    /// assert_eq!(vec.rank1(   20..   80), 1);
    /// assert_eq!(vec.rank1(   10..65535), 3);
    /// assert_eq!(vec.rank1(   10..65536), 3);
    /// assert_eq!(vec.rank1(   20..65579), 3);
    /// assert_eq!(vec.rank1(65536..65579), 1);
    /// assert_eq!(vec.rank1(65536..65580), 2);
    /// assert_eq!(vec.rank1(65536..     ), 2);
    /// ```
    #[inline(always)]
    pub fn rank1<I>(&self, index: I) -> u64
    where
        I: BitIndex<BitMap>,
    {
        index.rank1(self)
    }

    /// Returns the number of disabled bits within the given index.
    #[inline(always)]
    pub fn rank0<I>(&self, index: I) -> u64
    where
        I: BitIndex<BitMap>,
    {
        index.rank0(self)
    }

    #[inline(always)]
    pub fn select1(&self, n: u64) -> Option<u64> {
        BitSelect1::select1(self, n)
    }
    #[inline(always)]
    pub fn select0(&self, n: u64) -> Option<u64> {
        BitSelect0::select0(self, n)
    }

    pub fn and<R>(&self, rhs: R) -> And<&Self, R> {
        and(self, rhs)
    }
    pub fn and_not<R>(&self, rhs: R) -> AndNot<&Self, R> {
        and_not(self, rhs)
    }
    pub fn or<R>(&self, rhs: R) -> Or<&Self, R> {
        or(self, rhs)
    }
    pub fn xor<R>(&self, rhs: R) -> Xor<&Self, R> {
        xor(self, rhs)
    }
}

impl BitBlock for BitMap {
    const BITS: u64 = u32::CAP;
    fn empty() -> Self {
        Self::default()
    }
}

impl Bits for BitMap {
    #[inline]
    fn len(&self) -> u64 {
        Self::BITS
    }
    #[inline]
    fn count1(&self) -> u64 {
        Bits::make(&self.data).count1()
    }
}

impl Bits for BitMap {
    // Test bit at a given position.
    //
    // # Examples
    //
    // ```
    // use compacts::{bit::{Block, BitMap}, ops::Bits};
    // let map = BitMap::of(&[0, 80]);
    // assert!( map.get(0));
    // assert!(!map.get(1));
    // assert!( map.get(80));
    // assert!(!map.get(81));
    // assert!(!map.get(96));
    // ```
    //
    // # Panics
    //
    // Panics if index out of bounds.
    fn get(&self, i: u64) -> bool {
        assert!(i < self.len());
        let (i, o) = i.split(Block::BITS);
        self.block(i).map_or(false, |v| v.get(o))
    }

    fn getn<W: Word>(&self, i: u64, len: u64) -> W {
        assert!(len <= W::BITS && i < self.len() && i + len <= self.len());
        if len == 0 {
            return W::_0;
        }

        let j = i + len - 1;
        let (q0, r0) = i.split::<u16>(Block::BITS);
        let (q1, r1) = j.split::<u16>(Block::BITS);

        if q0 == q1 {
            let len = r1 - r0 + 1;
            self.block(q0).map_or(W::NONE, |v| v.getn(r0, len))
        } else {
            assert_eq!(q0 + 1, q1);
            let len = Block::BITS - r0;
            let head = self.block(q0).map_or(W::NONE, |v| v.getn(r0, len));
            let last = self.block(q1).map_or(W::NONE, |v| v.getn(0, r1 + 1));
            head | (last << try_cast(len))
        }
    }

    //    /// # Examples
    //    ///
    //    /// ```
    //    /// use compacts::{bit::BitMap, ops::Bits};
    //    /// let map = BitMap::of(&[0, 80]);
    //    /// assert_eq!(map.ones().collect::<Vec<_>>(), vec![0, 80]);
    //    /// ```
    //    fn ones<'a>(&'a self) -> Box<dyn Iterator<Item = u64> + 'a> {
    //        Box::new(self.keys.iter().enumerate().flat_map(move |(i, &k)| {
    //            let offset = try_cast::<u16, u64>(k) * Block::BITS;
    //            self.data[i].ones().map(move |b| b + offset)
    //        }))
    //    }
}

impl BitsMut for BitMap {
    fn put1(&mut self, i: u64) -> bool {
        assert!(i < self.len());
        let (index, offput) = i.split(Block::BITS);
        self.entry(&index).put1(offput)
    }

    fn put0(&mut self, i: u64) -> bool {
        assert!(i < self.len());
        let (index, offput) = i.split(Block::BITS);
        self.entry(&index).put0(offput)
    }

    fn putn<W: Word>(&mut self, i: u64, len: u64, word: W) {
        assert!(len <= W::BITS && i < self.len() && i + len <= self.len());
        if len == 0 {
            return;
        }

        let q0 = try_cast::<u64, u16>(i / Block::BITS);
        let q1 = try_cast::<u64, u16>((i + len - 1) / Block::BITS);
        if q0 == q1 {
            self.entry(&q0).putn(i % Block::BITS, len, word);
        } else {
            assert_eq!(q0 + 1, q1);
            let o = i % Block::BITS;
            let j = Block::BITS - o;

            let w = word.getn(0, j);
            self.entry(&q0).putn::<W>(o, j, w);
            let w = word.getn(j, len - j);
            self.entry(&q1).putn::<W>(0, len - j, w);
        }
    }
}

impl BitRank for BitMap {
    // Returns the number of enabled bits in the given range.
    //
    // # Examples
    //
    // ```
    // use compacts::{bit::BitMap, ops::{BitCount, BitRank}};
    // let map = BitMap::of(&[10, 20, 80, 65536, 65579]);
    // assert_eq!(map.rank1(   ..   ), map.count1());
    // # assert_eq!(map.rank1(0..0), 0);
    // # assert_eq!(map.rank1(1..1), 0);
    //
    // assert_eq!(map.rank1(   .. 80), 2);
    // assert_eq!(map.rank1(10 .. 80), 2);
    // assert_eq!(map.rank1(20 .. 80), 1);
    //
    // assert_eq!(map.rank1(10 .. 65535), 3);
    // assert_eq!(map.rank1(10 .. 65536), 3);
    // assert_eq!(map.rank1(20 .. 65579), 3);
    // assert_eq!(map.rank1(65536 .. 65579), 1);
    // assert_eq!(map.rank1(65536 .. 65580), 2);
    // ```
    //
    // # Panics
    //
    // Panics if index out of bounds.
    fn rank1(&self, i: u64, j: u64) -> u64 {
        let (q0, r0) = i.split(Block::BITS);
        let (q1, r1) = j.split(Block::BITS);
        match self.keys.binary_search(&try_cast(q0)) {
            Ok(n) | Err(n) if n < self.keys.len() => {
                if q0 == q1 {
                    self.data[n].rank1(r0, r1)
                } else {
                    let mut rank = 0;
                    for (j, &k) in self.keys[n..].iter().enumerate() {
                        let k = cast::<u16, u64>(k);
                        if k == q0 {
                            rank += self.data[j + n].rank1(r0, Block::BITS);
                        } else if k < q1 {
                            rank += self.data[j + n].count1(); // rank1(..)
                        } else if k == q1 {
                            rank += self.data[j + n].rank1(0, r1);
                            break;
                        }
                    }
                    rank
                }
            }
            _ => 0,
        }
    }
}

impl BitSelect1 for BitMap {
    fn select1(&self, mut n: u64) -> Option<u64> {
        for (i, v) in self.data.iter().enumerate() {
            let count = v.count1();
            if n < count {
                // remain < count implies that select1 never be None.
                let select1 = v.select1(n).expect("remain < count");
                return Some(try_cast::<u16, u64>(self.keys[i]) * Block::BITS + select1);
            }
            n -= count;
        }
        None
    }
}

impl BitSelect0 for BitMap {
    fn select0(&self, mut c: u64) -> Option<u64> {
        let mut prev: Option<u64> = None; // prev index
        for (i, value) in self.data.iter().enumerate() {
            let index = try_cast::<u16, u64>(self.keys[i]);

            let len = if let Some(p) = prev {
                // (p, index)
                index - (p + 1)
            } else {
                // [0, index)
                index
            };

            // None:    0..index
            // Some(p): p..index
            let count = value.count0() + Block::BITS * len;
            if c >= count {
                prev = Some(index);
                c -= count;
                continue;
            }

            // c < count
            let select0 = || {
                use std::iter::{once, repeat_with};

                let iter = repeat_with(|| None)
                    .take(try_cast::<u64, usize>(len))
                    .chain(once(Some(value)));

                // this block is almost same with [T]
                let mut remain = c;
                for (k, v) in iter.enumerate() {
                    let skipped_bit = try_cast::<usize, u64>(k) * Block::BITS;
                    let count0 = if let Some(v) = v {
                        v.count0()
                    } else {
                        Block::BITS
                    };
                    if remain < count0 {
                        return skipped_bit
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

            let skipped = prev.map_or(0, |p| (p + 1) * Block::BITS);
            return Some(skipped + select0());
        }

        let select = if let Some(&last) = self.keys.last() {
            (cast::<u16, u64>(last) + 1) * Block::BITS + c
        } else {
            c // empty
        };
        if select < self.len() {
            Some(select)
        } else {
            None
        }
    }
}

impl<'a, K: Word, V: Into<Block>> FromIterator<(K, V)> for BitMap {
    fn from_iter<I>(iterable: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
    {
        let mut keys = Vec::with_capacity(1 << 10);
        let mut data = Vec::with_capacity(1 << 10);
        iterable.into_iter().for_each(|(index, value)| {
            let index = try_cast(index);
            let block = value.into();
            keys.push(index);
            data.push(block);
        });
        keys.shrink_to_fit();
        data.shrink_to_fit();
        BitMap::new_unchecked(keys, data)
    }
}

impl<'a, K> FromIterator<(K, Cow<'a, Block>)> for BitMap
where
    K: Word,
{
    fn from_iter<I>(iterable: I) -> Self
    where
        I: IntoIterator<Item = (K, Cow<'a, Block>)>,
    {
        iterable
            .into_iter()
            .map(|(k, v)| (k, v.into_owned()))
            .collect()
    }
}

impl<'a> BitMask for &'a BitMap {
    type Index = u16;
    type Value = Cow<'a, Block>;
    type Steps = Steps<'a, u16>;
    fn into_steps(self) -> Self::Steps {
        assert_eq!(self.keys.len(), self.data.len());
        self.steps()
    }
}

pub struct Keys<'a, K> {
    iter: SliceIter<'a, K>,
}

pub struct Values<'a> {
    iter: SliceIter<'a, Block>,
}

pub struct Steps<'a, K> {
    zipped: Zip<Keys<'a, K>, Values<'a>>,
}

impl<'a, K> Iterator for Keys<'a, K> {
    type Item = &'a K;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<'a> Iterator for Values<'a> {
    type Item = &'a Block;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<'a, K: Word> Iterator for Steps<'a, K> {
    type Item = (K, Cow<'a, Block>);
    fn next(&mut self) -> Option<Self::Item> {
        self.zipped.find_map(|(&index, block)| {
            if block.any() {
                Some((index, Cow::Borrowed(block)))
            } else {
                None
            }
        })
    }
}

impl From<Bytes<&'_ [u8]>> for BitMap {
    fn from(bytes: Bytes<&'_ [u8]>) -> Self {
        bytes.into_steps().collect()
    }
}

const SERIAL_NO_RUN: u32 = 12346;
const SERIAL_COOKIE: u32 = 12347;
const NO_OFFSET_THRESHOLD: usize = 4;

impl BitMap {
    /// Assumed every block is not empty.
    pub fn serialize_into<W: io::Write>(&self, mut w: W) -> io::Result<()> {
        let blocks = self.keys.len();
        let runpos = {
            let mut runs = BitMap::from(vec![0u8; (blocks + 7) / 8]);
            for (i, Block(repr)) in self.data.iter().enumerate() {
                if repr.is_runs() {
                    runs.put1(try_cast(i));
                }
            }
            runs
        };
        let hasrun = runpos.count1() > 0;

        let mut offset = if hasrun {
            w.write_u16::<LE>(SERIAL_COOKIE as u16)?;
            w.write_u16::<LE>((blocks - 1) as u16)?;
            w.write_all(&runpos.0)?;
            if blocks < NO_OFFSET_THRESHOLD {
                4 + 4 * blocks + runpos.0.len()
            } else {
                4 + 8 * blocks + runpos.0.len()
            }
        } else {
            w.write_u32::<LE>(SERIAL_NO_RUN)?;
            w.write_u32::<LE>(blocks as u32)?;
            4 + 4 + 4 * blocks + 4 * blocks
        };

        for (&key, block) in self.keys.iter().zip(&self.data) {
            w.write_u16::<LE>(key)?;
            w.write_u16::<LE>((block.count1() - 1) as u16)?;
        }

        if !hasrun || blocks >= NO_OFFSET_THRESHOLD {
            for Block(repr) in &self.data {
                w.write_u32::<LE>(offset as u32)?;
                offset += repr.bytes_len();
            }
        }
        for Block(repr) in &self.data {
            repr.write_to(&mut w)?;
        }

        Ok(())
    }

    pub fn deserialize_from<R: io::Read>(mut r: R) -> io::Result<Self> {
        Header::read_from(&mut r).and_then(|desc| match desc {
            Header::Inline(map) => Ok(map),
            Header::Serial {
                runs, keys, pops, ..
            } => {
                let mut data = Vec::with_capacity(keys.len());
                for (i, pop) in pops.into_iter().enumerate() {
                    let pop = pop as usize;
                    let repr = if Bits::make(&runs).get(try_cast(i)) {
                        Repr::read_runs_from(&mut r, pop)?
                    } else if pop <= REPR_POS1_LEN {
                        Repr::read_heap_from(&mut r, pop)?
                    } else {
                        Repr::read_bits_from(&mut r, pop)?
                    };
                    data.push(Block(repr));
                }
                Ok(BitMap { keys, data })
            }

            Header::NoRuns { keys, pops, .. } => {
                let mut data = Vec::with_capacity(keys.len());
                for pop in pops {
                    let pop = pop as usize;
                    let repr = if pop <= REPR_POS1_LEN {
                        Repr::read_heap_from(&mut r, pop)?
                    } else {
                        Repr::read_bits_from(&mut r, pop)?
                    };
                    data.push(Block(repr));
                }
                Ok(BitMap { keys, data })
            }
        })
    }
}

impl<'a> Bytes<&'a [u8]> {
    pub fn new(bytes: &'a [u8]) -> io::Result<Self> {
        let header = Header::from_bytes(bytes)?;
        Ok(Bytes { header, bytes })
    }
}

impl Bytes<&'_ [u8]> {
    fn blocks(&self) -> usize {
        match self.header {
            Header::Inline(ref map) => map.keys.len(),
            Header::Serial { ref keys, .. } => keys.len(),
            Header::NoRuns { ref keys, .. } => keys.len(),
        }
    }

    fn step(&self, i: usize) -> (u16, Cow<Block>) {
        match self.header {
            Header::Inline(ref map) => (map.keys[i], Cow::Borrowed(&map.data[i])),

            Header::Serial {
                ref runs,
                ref keys,
                ref pops,
                ref locs,
            } => {
                let pop = pops[i] as usize;
                let loc = locs[i] as usize;
                (
                    keys[i],
                    Cow::Owned(Block(if Bits::make(&runs).get(try_cast(i)) {
                        Repr::runs_from_bytes(&self.bytes[loc..], pop)
                    } else if pop <= REPR_POS1_LEN {
                        Repr::heap_from_bytes(&self.bytes[loc..], pop)
                    } else {
                        Repr::bits_from_bytes(&self.bytes[loc..], pop)
                    })),
                )
            }

            Header::NoRuns {
                ref keys,
                ref pops,
                ref locs,
            } => {
                let pop = pops[i] as usize;
                let loc = locs[i] as usize;
                (
                    keys[i],
                    Cow::Owned(Block(if pop <= REPR_POS1_LEN {
                        Repr::heap_from_bytes(&self.bytes[loc..], pop)
                    } else {
                        Repr::bits_from_bytes(&self.bytes[loc..], pop)
                    })),
                )
            }
        }
    }
}

impl Header {
    fn runpos<T: io::Read>(mut r: T, blocks: usize) -> io::Result<Vec<u8>> {
        let mut buf = vec![0; (blocks + 7) / 8];
        r.read_exact(&mut buf).map(|()| buf)
    }

    fn header<T: io::Read>(mut r: T, blocks: usize) -> io::Result<(Vec<u16>, Vec<u32>)> {
        let mut keys = Vec::with_capacity(blocks);
        let mut pops = Vec::with_capacity(blocks);
        for _ in 0..blocks {
            let key = r.read_u16::<LE>()?;
            let pop = cast::<u16, u32>(r.read_u16::<LE>()?) + 1;
            keys.push(key);
            pops.push(pop);
        }
        Ok((keys, pops))
    }

    fn offsets<T: io::Read>(mut r: T, blocks: usize) -> io::Result<Vec<u32>> {
        let mut offsets = vec![0u32; blocks];
        r.read_u32_into::<LE>(&mut offsets).map(|()| offsets)
    }

    fn from_bytes(data: &[u8]) -> io::Result<Self> {
        Self::read_from(io::Cursor::new(data))
    }

    fn read_from<R: io::Read>(mut r: R) -> io::Result<Self> {
        match r.read_u32::<LE>()? {
            cookie if cookie & 0x0000_FFFF == SERIAL_COOKIE => {
                // BitMap has `Repr::Runs`.
                let blocks = ((cookie >> 16u32) as usize) + 1; // number of blocks

                let runs = Self::runpos(&mut r, blocks)?;
                let (keys, pops) = Self::header(&mut r, blocks)?;
                assert_eq!(keys.len(), blocks);
                assert_eq!(pops.len(), blocks);
                if blocks < NO_OFFSET_THRESHOLD {
                    let mut data = Vec::with_capacity(blocks);
                    for (i, pop) in pops.into_iter().enumerate() {
                        let pop = pop as usize;
                        let repr = if Bits::make(&runs).get(try_cast(i)) {
                            Repr::read_runs_from(&mut r, pop)?
                        } else if pop <= REPR_POS1_LEN {
                            Repr::read_heap_from(&mut r, pop)?
                        } else {
                            Repr::read_bits_from(&mut r, pop)?
                        };
                        data.push(Block(repr));
                    }
                    Ok(Header::Inline(BitMap { keys, data }))
                } else {
                    let locs = Self::offsets(&mut r, blocks)?;
                    assert_eq!(locs.len(), blocks);
                    Ok(Header::Serial {
                        runs,
                        keys,
                        pops,
                        locs,
                    })
                }
            }

            cookie if cookie == SERIAL_NO_RUN => {
                let blocks = r.read_u32::<LE>()? as usize;

                let (keys, pops) = Self::header(&mut r, blocks)?;
                let locs = Self::offsets(&mut r, blocks)?;
                assert_eq!(keys.len(), blocks);
                assert_eq!(pops.len(), blocks);
                assert_eq!(locs.len(), blocks);
                Ok(Header::NoRuns { keys, pops, locs })
            }

            _ => Err(io::Error::new(io::ErrorKind::Other, "unknown cookie")),
        }
    }
}

pub struct BytesSteps<'bytes, 'b> {
    index: std::ops::Range<usize>,
    bytes: &'b Bytes<&'bytes [u8]>,
}

impl<'bytes, 'b> BitMask for &'b Bytes<&'bytes [u8]> {
    type Index = u16;
    type Value = Cow<'b, Block>;
    type Steps = BytesSteps<'bytes, 'b>;
    fn into_steps(self) -> Self::Steps {
        let index = 0..self.blocks();
        let bytes = self;
        BytesSteps { index, bytes }
    }
}

impl<'bytes, 'b> Iterator for BytesSteps<'bytes, 'b> {
    type Item = (u16, Cow<'b, Block>);
    fn next(&mut self) -> Option<Self::Item> {
        self.index.next().map(|i| self.bytes.step(i))
    }
}

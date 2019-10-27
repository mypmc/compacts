////! `map`

use std::{
    borrow::Cow,
    iter::{Enumerate, FromIterator},
    ops::RangeBounds,
    slice,
};

use crate::{
    bits::{bit_vec::BitVec, blocks_by, Mask, Words},
    fenwick::FenwickTree,
    num::Word,
    ops::*,
};

/// `BitMap<T>`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BitMap<T: Words> {
    tree: FenwickTree<usize>,     // prefix sum of `1`.
    bits: BitVec<Option<Box<T>>>, // bit blocks, bits.size() is the capacity of `BitMap`
}

impl<T: Words> BitMap<T> {
    /// Reserves specified capacity by multiples of T::SIZE, such that `BitMap` has at least `n` bits.
    pub fn none(n: usize) -> Self {
        let bits = BitVec::from_fn(n, || None);
        let tree = FenwickTree::with_default(blocks_by(n, T::BITS));
        BitMap { tree, bits }
    }

    // // pub(crate) fn from_buf(bits: Vec<Option<Box<T>>>, size: usize) -> Self {
    // //     assert!(size <= bits.size());
    // //     let mut tree = Fenwick::new(bits.len());
    // //     for (i, o) in bits.iter().enumerate() {
    // //         tree.add(i, o.count1());
    // //     }
    // //     BitMap { tree, size, bits }
    // // }

    // #[inline]
    // pub fn capacity(&self) -> usize {
    //     self.bits.capacity() // allocated bits
    // }

    // #[inline]
    // pub fn len(&self) -> usize {
    //     self.bits.len()
    // }

    // pub unsafe fn set_len(&mut self, new_size: usize) {
    //     // like thie
    //     // self.bits.set_len()
    //     debug_assert!(new_size <= self.capacity());
    //     self.size = new_size;
    // }

    // fn push(&mut self, b: bool) {
    //     let i = self.size;
    //     if self.bits.size() <= i {
    //         self.bits.resize(blocks(i, T::SIZE), None);
    //     }
    //     if b {
    //         self.bits.put1(i);
    //     }
    //     // else {
    //     //     self.bits.put0(i);
    //     // }
    //     self.size += 1;
    // }
}

impl<T: Words> Bits for BitMap<T> {
    #[inline]
    fn size(&self) -> usize {
        self.bits.size()
    }

    #[inline]
    fn bit(&self, i: usize) -> bool {
        self.bits.bit(i)
    }

    #[inline]
    fn getn<W: Word>(&self, i: usize, n: usize) -> W {
        self.bits.getn(i, n)
    }

    #[inline]
    fn count1(&self) -> usize {
        self.tree.sum(self.bits.buf.len())
    }

    /// ```
    /// use compacts::{BitMap, ops::{BitsMut, Bits}};
    /// let mut bv = BitMap::<[u64; 1024]>::none(1000);
    /// bv.put1(10);
    /// bv.put1(50);
    /// bv.put1(100);
    /// assert_eq!(bv.rank1(..100), 2);
    /// ```
    #[inline]
    fn rank1<R: RangeBounds<usize>>(&self, range: R) -> usize {
        let rank = |p: usize| {
            if p == self.size() {
                self.count1()
            } else {
                let (q, r) = divrem!(p, T::BITS);
                self.tree.sum::<usize>(q) + self.bits.buf[q].rank1(..r)
            }
        };
        match super::to_exclusive(&range, self.size()).expect("out of bounds") {
            (0, i) => rank(i),
            (i, j) => rank(j) - rank(i),
        }
    }

    /// ```
    /// use compacts::{BitMap, ops::{BitsMut, Bits}};
    /// let mut bv = BitMap::<[u64; 1024]>::none(66666);
    /// bv.put1(10);
    /// bv.put1(50);
    /// bv.put1(100);
    /// bv.put1(65535);
    /// bv.put1(65536);
    /// assert_eq!(bv.select1(0), Some(10));
    /// assert_eq!(bv.select1(1), Some(50));
    /// assert_eq!(bv.select1(2), Some(100));
    /// assert_eq!(bv.select1(3), Some(65535));
    /// assert_eq!(bv.select1(4), Some(65536));
    /// assert_eq!(bv.select1(5), None);
    /// ```
    #[inline]
    fn select1(&self, n: usize) -> Option<usize> {
        self.tree.search(n + 1).ok().map(|i| {
            let offset = i * T::BITS;
            let remain = n - self.tree.sum::<usize>(i);
            offset + self.bits.buf[i].select1(remain).unwrap()
        })
    }
}

impl<T: Words> BitsMut for BitMap<T> {
    fn put1(&mut self, i: usize) {
        BOUNDS_CHECK!(i < self.size());
        let (i, o) = divrem!(i, T::BITS);
        let buf = &mut self.bits.buf;
        if !buf[i].bit(o) {
            buf[i].put1(o);
            self.tree.add(i, 1);
        }
    }

    fn put0(&mut self, i: usize) {
        BOUNDS_CHECK!(i < self.size());
        let (i, o) = divrem!(i, T::BITS);
        let buf = &mut self.bits.buf;
        if buf[i].bit(o) {
            buf[i].put0(o);
            self.tree.sub(i, 1);
        }
    }

    fn flip(&mut self, i: usize) {
        assert!(i < self.size());
        let (i, o) = divrem!(i, T::BITS);
        let buf = &mut self.bits.buf;
        if buf[i].bit(o) {
            buf[i].put0(o);
            self.tree.sub(i, 1);
        } else {
            buf[i].put1(o);
            self.tree.add(i, 1);
        }
    }
}

impl<'a, T: Words> Mask<'a> for &'a BitMap<T> {
    type Block = [T::Word];
    type Steps = Steps<'a, T>;
    fn into_steps(self) -> Self::Steps {
        Steps(Iter {
            // TODO
            iter: self.bits.buf.iter().enumerate(),
        })
    }
}

/// `Mask::Steps` for `BitMap`.
#[derive(Debug, Clone)]
pub struct Steps<'a, T: Words>(Iter<'a, T>);

#[derive(Debug, Clone)]
struct Iter<'a, T: Words> {
    iter: Enumerate<slice::Iter<'a, Option<Box<T>>>>,
}

impl<'a, T: Words> Iterator for Steps<'a, T> {
    type Item = (usize, Cow<'a, [T::Word]>);
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(i, s)| (i, Cow::Borrowed(s)))
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a, T: Words> Iterator for Iter<'a, T> {
    type Item = (usize, &'a [T::Word]);
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .find_map(|(i, opt)| opt.as_ref().map(|s| (i, s.as_ref_words())))
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, T: Words> FromIterator<(usize, Cow<'a, [T::Word]>)> for BitMap<T> {
    fn from_iter<I>(iterable: I) -> Self
    where
        I: IntoIterator<Item = (usize, Cow<'a, [T::Word]>)>,
    {
        let mut len = 0;
        let mut buf = Vec::new();
        for (index, cow) in iterable {
            if index > buf.len() {
                buf.resize(index, None);
            }

            buf.insert(index, {
                let mut arr = T::none();
                arr.as_mut_words().copy_from_slice(cow.as_ref());
                Some(Box::new(arr))
            });

            len += T::BITS;
        }

        buf.shrink_to_fit();

        let mut tree = FenwickTree::with_default(buf.len());
        for (i, p) in buf.iter().enumerate() {
            tree.add(i, p.count1());
        }

        let bits = BitVec { buf, len };
        BitMap { tree, bits }
    }
}

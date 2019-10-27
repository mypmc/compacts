use std::{iter, ops::RangeBounds};

use crate::{
    bits::{blocks_by, to_exclusive},
    ops::*,
};

/// `BitVec<B>`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BitVec<B> {
    pub(crate) buf: Vec<B>, // bit blocks
    pub(crate) len: usize,  // bit length
}

// /// `IntVec<B>`
// #[derive(Debug, Clone, PartialEq, Eq)]
// pub struct IntVec<B, C: Coder> {
//     size: usize, // int's bit size
//     buf: BitVec<B>,
// }

impl<B> Default for BitVec<B> {
    fn default() -> Self {
        BitVec {
            buf: Vec::new(),
            len: 0,
        }
    }
}

impl<B: FixedBits> BitVec<B> {
    /// Returns an empty `BitVec`.
    ///
    /// ```
    /// let bv = compacts::BitVec::<u64>::new();
    /// assert!(bv.is_empty() && bv.capacity() == 0);
    /// ```
    pub fn new() -> Self {
        BitVec {
            buf: Vec::new(),
            len: 0,
        }
    }

    /// Returns an empty `BitVec` with the at least specified capacity.
    ///
    /// ```
    /// let bv = compacts::BitVec::<u64>::with_capacity(10);
    /// assert!(bv.is_empty() && bv.capacity() >= 10);
    /// ```
    pub fn with_capacity(cap: usize) -> Self {
        BitVec {
            buf: Vec::with_capacity(blocks_by(cap, B::SIZE)),
            len: 0,
        }
    }

    /// Returns a zeroed `BitVec` with the specified length.
    ///
    /// ```
    /// let bv = compacts::BitVec::<u64>::none(100);
    /// assert!(bv.len() == 100 && bv.capacity() >= 100);
    /// ```
    pub fn none(len: usize) -> Self {
        Self::from_fn(len, B::none)
    }

    /// Allocates buf by multiples of `B::SIZE`, such that `BitVec` has at least `n` length and capacity.
    ///
    /// ```
    /// let bv = compacts::BitVec::<u64>::from_fn(1000, || !0);
    /// assert!(bv.len() == 1000 && bv.capacity() >= 1000);
    /// ```
    pub fn from_fn<F>(len: usize, mut f: F) -> Self
    where
        F: FnMut() -> B,
    {
        BitVec {
            buf: iter::from_fn(|| Some(f()))
                .take(blocks_by(len, B::SIZE))
                .collect(),
            len,
        }
    }

    /// ```
    /// let bv = compacts::BitVec::<u64>::of(vec![0, 1, 3, 5]);
    /// assert!( bv.bit(0));
    /// assert!( bv.bit(1));
    /// assert!(!bv.bit(2));
    /// assert!( bv.bit(3));
    /// assert!(!bv.bit(4));
    /// assert!( bv.bit(5));
    /// ```
    pub fn of<A: AsRef<[usize]>>(slice: A) -> Self {
        let slice = slice.as_ref();
        let mut bv = Self::with_capacity(slice.len());
        for &b in slice {
            if bv.len <= b {
                bv.resize(b + 1);
            }
            bv.put1(b);
        }
        bv.shrink_to_fit();
        bv
    }

    /// Returns the number of buf the buftor can hold without reallocating.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.buf.capacity() * B::SIZE
    }

    /// Returns the number of buf.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the buftor contains no buf.
    ///
    /// ```
    /// let mut bv = compacts::BitVec::<u64>::with_capacity(10);
    /// assert!(bv.is_empty());
    /// bv.push(true);
    /// assert!(!bv.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Reserves capacity by multiples of `B::SIZE` for at least `additional` more buf to be inserted.
    ///
    /// After calling `reserve`, capacity will be greater than or equal to `self.len() + additional`.
    /// Does nothing if capacity is already sufficient.
    ///
    /// ```
    /// let mut bv = compacts::BitVec::<u64>::with_capacity(0);
    /// bv.reserve(100);
    /// assert!(bv.capacity() >= 128);
    /// ```
    pub fn reserve(&mut self, additional: usize) {
        let len = blocks_by(self.len + additional, B::SIZE);
        self.buf.reserve(len - self.buf.len());
    }

    /// Resizes the `BitVec` in-place so that `len` is equal to `new_len`.
    ///
    /// ```
    /// let mut bv = compacts::BitVec::<u64>::with_capacity(0);
    /// bv.resize(100);
    /// assert!(bv.len() == 100 && dbg!(bv.capacity()) >= 100);
    /// bv.resize(10);
    /// assert!(bv.len() == 10  && bv.capacity() >= 100);
    /// ```
    pub fn resize(&mut self, new_len: usize) {
        self.resize_with(new_len, B::none)
    }

    /// Resizes the `BitVec` in-place so that `len` is equal to `new_len`.
    ///
    /// If `new_len` is greater than `len`, the `Vec` is extended by the difference,
    /// with each additional block `B` filled with the result of calling the closure `f`.
    ///
    /// ```
    /// let mut bv = compacts::BitVec::<u64>::with_capacity(0);
    /// let mut p = 2u64;
    /// bv.resize_with(100, || { p *= 2; p });
    /// ```
    pub fn resize_with<F: FnMut() -> B>(&mut self, new_len: usize, f: F) {
        if self.len < new_len {
            self.buf.resize_with(blocks_by(new_len, B::SIZE), f);
            self.len = new_len;
        } else if new_len < self.len {
            self.truncate(new_len);
        }
    }

    /// Shortens the buf, keeping the first `len` buf and dropping the rest.
    ///
    /// Note that this method has no effect on the allocated capacity.
    ///
    /// ```
    /// let mut bv = compacts::BitVec::<u64>::with_capacity(10);
    /// for i in 0..10 {
    ///     bv.push(i % 2 == 0);
    /// }
    ///
    /// bv.truncate(3);
    /// assert_eq!(bv.len(), 3);
    /// ```
    #[inline]
    pub fn truncate(&mut self, len: usize) {
        if len < self.len {
            self.len = len;
            self.buf.truncate(blocks_by(self.len, B::SIZE));
        }
    }

    /// Shrinks the capacity as much as possible.
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.buf.truncate(blocks_by(self.len, B::SIZE));
        self.buf.shrink_to_fit();
    }

    // pub fn insert(&mut self, index: usize, b: T) {
    //     shiftr(i, 1)
    // }

    // pub fn remove(&mut self, index: usize) -> bool {
    //     shiftl(i, 1)
    // }

    /// ```
    /// let v = compacts::BitVec::<u64>::from_fn(1000, ||  0);
    /// let w = compacts::BitVec::<u64>::from_fn(1000, || !0);
    /// assert_eq!(v.count1(), 0);
    /// assert_eq!(w.count1(), 1000);
    /// ```
    #[inline]
    pub fn count1(&self) -> usize {
        Bits::count1(self)
    }

    /// ```
    /// let v = compacts::BitVec::<u64>::from_fn(1000, ||  0);
    /// let w = compacts::BitVec::<u64>::from_fn(1000, || !0);
    /// assert_eq!(v.count0(), 1000);
    /// assert_eq!(w.count0(), 0);
    /// ```
    #[inline]
    pub fn count0(&self) -> usize {
        Bits::count0(self)
    }

    /// ```
    /// let v = compacts::BitVec::<u64>::from_fn(1000, ||  0);
    /// let w = compacts::BitVec::<u64>::from_fn(1000, || !0);
    /// assert!(!v.all());
    /// assert!( w.all());
    /// ```
    #[inline]
    pub fn all(&self) -> bool {
        Bits::all(self)
    }

    /// ```
    /// let v = compacts::BitVec::<u64>::from_fn(1000, ||  0);
    /// let w = compacts::BitVec::<u64>::from_fn(1000, || !0);
    /// assert!(!v.any());
    /// assert!( w.any());
    /// ```
    #[inline]
    pub fn any(&self) -> bool {
        Bits::any(self)
    }

    /// ```
    /// let v = compacts::BitVec::<u64>::from_fn(1000, ||  0);
    /// let w = compacts::BitVec::<u64>::from_fn(1000, || !0);
    /// assert!(!v.any());
    /// assert!( w.any());
    /// ```
    #[inline]
    pub fn bit(&self, i: usize) -> bool {
        Bits::bit(self, i)
    }

    #[inline]
    pub fn put(&mut self, i: usize, b: bool) {
        BitsMut::put(self, i, b);
    }

    /// Appends a bit to the back of a collection.
    ///
    /// ```
    /// let mut bv = compacts::BitVec::<u64>::with_capacity(10);
    /// bv.push(false);
    /// bv.push(true);
    /// assert_eq!(bv.len(), 2);
    /// assert_eq!(bv.pop(), Some(true));
    /// assert_eq!(bv.pop(), Some(false));
    /// assert_eq!(bv.len(), 0);
    /// ```
    pub fn push(&mut self, b: bool) {
        if self.len == self.buf.len() * B::SIZE {
            self.buf.push(B::none());
        }
        if b {
            self.buf.put1(self.len);
        }
        // else {
        //     self.buf.put0(i);
        // }
        self.len += 1;
    }

    /// Removes the last bit and returns it.
    pub fn pop(&mut self) -> Option<bool> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            Some(self.buf.bit(self.len))
        }
    }

    /// Swaps bit at `i` by `bit` and returns the previous value.
    #[inline]
    pub fn swap(&mut self, i: usize, bit: bool) -> bool {
        BOUNDS_CHECK!(i < self.len);
        let (i, o) = divrem!(i, B::SIZE);
        let cur = self.buf[i].bit(o);
        if !cur && bit {
            self.buf[i].put1(o);
        } else if cur && !bit {
            self.buf[i].put0(o);
        }
        cur
    }

    // /// ```
    // /// let v = compacts::BitVec::<u64>::from_buf(vec![0, 100, 1000, 10000]);
    // /// assert_eq!(v.rank1(0..0), 0);
    // /// assert_eq!(v.rank1(..10000), 3);
    // /// ```
    // #[inline]
    // pub fn rank1<R: RangeBounds<usize>>(&self, range: R) -> usize {
    //     self.buf(range).count1()
    // }

    // #[inline]
    // pub fn rank0<R: RangeBounds<usize>>(&self, range: R) -> usize {
    //     self.buf(range).count0()
    // }

    // fn buf_mut<R: RangeBounds<usize>>(&mut self, range: R) -> BitVecMut<'_, B> {
    //     let (beg, end) = crate::buf::bounds(&range, B::SIZE);
    //     assert!(beg <= end && end <= self.len);
    //     let buf = &mut self.buf;
    //     BitVecMut { beg, end, buf }
    // }
}

// impl<'a, B> BitVecMut<'a, B> {
//     #[inline]
//     fn as_buf(&self) -> BitVec<'a, B> {
//         BitVec {
//             beg: self.beg,
//             end: self.end,
//             buf: &*self.buf,
//         }
//     }
// }

impl<B: FixedBits> Bits for BitVec<B> {
    #[inline]
    fn size(&self) -> usize {
        self.len
    }

    #[inline]
    fn bit(&self, i: usize) -> bool {
        BOUNDS_CHECK!(i < self.len);
        self.buf.bit(i)
    }

    #[inline]
    fn count1(&self) -> usize {
        self.buf.rank1(..self.len)
        // let (q, r) = divrem!(self.len, B::SIZE);
        // self.buf[..q].count1() + self.buf.get(q).map_or(0, |p| p.rank1(..r))
    }

    fn rank1<R: RangeBounds<usize>>(&self, range: R) -> usize {
        let (i, j) = to_exclusive(&range, self.len).expect("out of bounds");

        let (q0, r0) = divrem!(i, B::SIZE);
        let (q1, r1) = divrem!(j, B::SIZE);
        if q0 == q1 {
            self.buf[q0].rank1(r0..r1)
        } else {
            self.buf[q0].rank1(r0..)
                + self.buf[q0 + 1..q1].count1()
                + self.buf.get(q1).map_or(0, |b| b.rank1(..r1))
        }
    }

    #[inline]
    fn all(&self) -> bool {
        let (q, r) = divrem!(self.len, B::SIZE);
        self.buf[..q].all() && self.buf.get(q).map_or(true, |p| p.rank0(..r) == 0)
    }

    #[inline]
    fn any(&self) -> bool {
        let (q, r) = divrem!(self.len, B::SIZE);
        self.buf[..q].any() || self.buf.get(q).map_or(false, |p| p.rank1(..r) > 0)
    }
}

impl<B: FixedBits> BitsMut for BitVec<B> {
    #[inline]
    fn put1(&mut self, i: usize) {
        BOUNDS_CHECK!(i < self.len);
        self.buf.put1(i);
    }
    #[inline]
    fn put0(&mut self, i: usize) {
        BOUNDS_CHECK!(i < self.len);
        self.buf.put0(i);
    }
    #[inline]
    fn flip(&mut self, i: usize) {
        BOUNDS_CHECK!(i < self.len);
        self.buf.flip(i);
    }
}

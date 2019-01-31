use std::{
    borrow::Cow,
    fmt,
    ops::{self, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Range},
};

use crate::bit::{self, ops::*, ucast, UnsignedInt};

#[derive(Clone, Eq)]
pub struct Block<A: BlockArray> {
    ones: u32,
    data: Option<Box<A>>,
}

// impl<A: BlockArray, Idx: SliceIndex<[A::Value]>> ops::Index<Idx> for Block<A> {
//     type Output = <Idx as SliceIndex<[A::Value]>>::Output;
//     fn index(&self, index: Idx) -> &Self::Output {
//         index.index(self.as_ref().expect(""))
//     }
// }

impl<A: BlockArray> ops::Index<usize> for Block<A> {
    type Output = A::Value;
    fn index(&self, i: usize) -> &Self::Output {
        static MSG: &str = "index out of bounds: not allocated block";
        &self.as_ref().expect(MSG)[i]
    }
}

// impl<A: BlockArray> ops::IndexMut<usize> for Block<A> {
//     fn index_mut(&mut self, index: usize) -> &mut Self::Output {
//         &mut self.alloc().as_slice_mut()[index]
//     }
// }

pub trait BlockArray:
    crate::private::Sealed
    + Copy
    + FiniteBits
    + Access
    + Rank
    + Select1
    + Select0
    + Assign<u64>
    + Assign<Range<u64>, Output = u64>
{
    type Value: UnsignedInt;
    const LEN: usize;

    fn splat(value: Self::Value) -> Self;

    fn as_slice(&self) -> &[Self::Value];

    fn as_slice_mut(&mut self) -> &mut [Self::Value];
}

impl<A: BlockArray> Default for Block<A> {
    fn default() -> Self {
        Block {
            ones: 0,
            data: None,
        }
    }
}

impl<A: BlockArray> fmt::Debug for Block<A>
where
    A::Value: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(slice) = self.as_ref() {
            f.debug_list().entries(slice).finish()
        } else {
            f.pad("Block")
        }
    }
}

impl<A: BlockArray> PartialEq for Block<A> {
    /// # Examples
    ///
    /// ```
    /// use compacts::bit::{Block, ops::FiniteBits};
    /// let a = Block::<[u64; 1024]>::empty();
    /// let b = Block::<[u64; 1024]>::empty();
    /// let c = Block::<[u64; 1024]>::splat(0);
    /// assert_eq!(a, b);
    /// assert_eq!(a, c);
    /// assert_eq!(c, a);
    /// assert_eq!(c, b);
    /// let d = Block::<[u64; 1024]>::splat(3);
    /// let e = Block::<[u64; 1024]>::splat(3);
    /// assert_eq!(d, e);
    /// ```
    fn eq(&self, that: &Block<A>) -> bool {
        if self.ones != that.ones {
            return false;
        }

        // if both `ones` are zero, we do not care about its data representation
        (self.ones == 0 && that.ones == 0) || self.as_ref() == that.as_ref()
    }
}

impl<A: BlockArray> From<A> for Block<A> {
    fn from(array: A) -> Self {
        let ones = ucast(array.count1());
        let data = Some(Box::new(array));
        Block { ones, data }
    }
}
impl<A: BlockArray> From<&'_ A> for Block<A> {
    fn from(array: &A) -> Self {
        let ones = ucast(array.count1());
        let data = Some(Box::new(*array));
        Block { ones, data }
    }
}

impl<A: BlockArray> Block<A> {
    /// Constructs a new instance with each element initialized to value.
    ///
    /// # Examples
    ///
    /// ```
    /// use compacts::bit::Block;
    /// let _ = Block::<[u64; 1024]>::splat(0b_00010001);
    /// ```
    pub fn splat(value: A::Value) -> Self {
        let ones = ucast::<u64, u32>(value.count1()) * ucast::<usize, u32>(A::LEN);
        let data = Some(Box::new(A::splat(value)));
        Block { ones, data }
    }

    pub fn as_ref(&self) -> Option<&[A::Value]> {
        self.data.as_ref().map(|a| a.as_slice())
    }
    pub fn as_mut(&mut self) -> Option<&mut [A::Value]> {
        self.data.as_mut().map(|a| a.as_slice_mut())
    }

    pub fn copy_from_slice<T: AsRef<[A::Value]>>(&mut self, data: T) {
        let this = self.alloc().as_slice_mut();
        let that = data.as_ref();
        this[..that.len()].copy_from_slice(that);
        self.ones = ucast(this.count1());
    }

    fn alloc(&mut self) -> &mut A {
        if self.data.is_none() {
            *self = Self::splat(A::Value::ZERO);
        }
        self.data.as_mut().unwrap()
    }
}

impl<A: BlockArray> FiniteBits for Block<A> {
    const BITS: u64 = A::Value::BITS * A::LEN as u64;
    fn empty() -> Self {
        Self::default()
    }
}

impl<A: BlockArray> Count for Block<A> {
    fn bits(&self) -> u64 {
        Self::BITS
    }
    fn count1(&self) -> u64 {
        u64::from(self.ones)
    }
}

impl<A: BlockArray> Access for Block<A> {
    fn access(&self, i: u64) -> bool {
        self.data.as_ref().map_or(false, |a| a.access(i))
    }

    fn iterate<'a>(&'a self) -> Box<dyn Iterator<Item = u64> + 'a> {
        if let Some(a) = self.data.as_ref() {
            a.iterate()
        } else {
            Box::new(std::iter::empty())
        }
    }
}

impl<A: BlockArray> Rank for Block<A> {
    fn rank1(&self, i: u64) -> u64 {
        self.data.as_ref().map_or(0, |a| a.rank1(i))
    }
}

impl<A: BlockArray> Select1 for Block<A> {
    fn select1(&self, n: u64) -> Option<u64> {
        self.data.as_ref().and_then(|a| a.select1(n))
    }
}
impl<A: BlockArray> Select0 for Block<A> {
    fn select0(&self, n: u64) -> Option<u64> {
        self.data.as_ref().map_or(Some(n), |a| a.select0(n))
    }
}

impl<A: BlockArray> Assign<u64> for Block<A> {
    type Output = ();

    fn set1(&mut self, i: u64) -> Self::Output {
        assert!(i < self.bits(), bit::OUT_OF_BOUNDS);
        if !self.access(i) {
            let arr = self.alloc();
            arr.set1(i);
            self.ones += 1;
        }
    }
    fn set0(&mut self, i: u64) -> Self::Output {
        assert!(i < self.bits(), bit::OUT_OF_BOUNDS);
        if self.access(i) {
            let arr = self.alloc();
            arr.set0(i);
            self.ones -= 1;
        }
    }
}

impl<A: BlockArray> Assign<Range<u64>> for Block<A> {
    type Output = <A as Assign<Range<u64>>>::Output;

    /// # Examples
    ///
    /// ```
    /// use compacts::bit::{Block, ops::{FiniteBits, Assign}};
    /// let mut block = Block::<[u8; 8192]>::empty();
    /// assert_eq!(block.as_ref(), None);
    /// assert_eq!(block.set1(0..3), 3);
    /// assert_eq!(&block.as_ref().unwrap()[..3], &[0b_00000111u8, 0b_00000000, 0b_00000000]);
    /// assert_eq!(block.set1(14..18), 4);
    /// assert_eq!(&block.as_ref().unwrap()[..3], &[0b_00000111u8, 0b_11000000, 0b_00000011]);
    /// ```
    fn set1(&mut self, i: Range<u64>) -> Self::Output {
        let arr = self.alloc();
        let out = arr.set1(i);
        self.ones += ucast::<u64, u32>(out);
        out
    }
    fn set0(&mut self, i: Range<u64>) -> Self::Output {
        if let Some(arr) = self.data.as_mut() {
            let out = arr.set0(i);
            self.ones -= ucast::<u64, u32>(out);
            out
        } else {
            0u64
        }
    }
}

impl<A: BlockArray> BitAnd<&'_ Block<A>> for &'_ Block<A> {
    type Output = Block<A>;
    fn bitand(self, that: &Block<A>) -> Self::Output {
        match (self.data.as_ref(), that.data.as_ref()) {
            (Some(lhs), Some(rhs)) => {
                let mut data = A::empty();
                let mut ones = 0;
                let mapped = lhs
                    .as_slice()
                    .iter()
                    .zip(rhs.as_slice())
                    .map(|(&a, &b)| a & b);

                for (x, y) in data.as_slice_mut().iter_mut().zip(mapped) {
                    *x = y;
                    ones += x.count1();
                }
                Block {
                    ones: ucast(ones),
                    data: Some(Box::new(data)),
                }
            }
            _ => Block::empty(),
        }
    }
}

impl<A: BlockArray> BitAndAssign<&'_ Block<A>> for Block<A> {
    fn bitand_assign(&mut self, that: &Block<A>) {
        match (self.data.as_mut(), that.data.as_ref()) {
            (Some(lhs), Some(rhs)) => {
                let mut ones = 0;
                for (x, y) in lhs.as_slice_mut().iter_mut().zip(rhs.as_slice()) {
                    *x &= *y;
                    ones += x.count1();
                }
                self.ones = ucast(ones);
            }
            _ => {
                self.data = None;
            }
        }
    }
}

impl<A: BlockArray> BitOr<&'_ Block<A>> for &'_ Block<A> {
    type Output = Block<A>;

    fn bitor(self, that: &Block<A>) -> Self::Output {
        match (self.data.as_ref(), that.data.as_ref()) {
            (Some(lhs), Some(rhs)) => {
                let mut data = A::empty();
                let mut ones = 0;
                let slice = lhs
                    .as_slice()
                    .iter()
                    .zip(rhs.as_slice())
                    .map(|(&a, &b)| a | b);

                for (x, y) in data.as_slice_mut().iter_mut().zip(slice) {
                    *x = y;
                    ones += x.count1();
                }
                Block {
                    ones: ucast(ones),
                    data: Some(Box::new(data)),
                }
            }
            (Some(lhs), None) => Block {
                ones: self.ones,
                data: Some(lhs.clone()),
            },
            (None, Some(rhs)) => Block {
                ones: self.ones,
                data: Some(rhs.clone()),
            },
            _ => Block::empty(),
        }
    }
}

impl<A: BlockArray> BitOrAssign<&'_ Block<A>> for Block<A> {
    fn bitor_assign(&mut self, that: &Block<A>) {
        match (self.data.as_mut(), that.data.as_ref()) {
            (Some(lhs), Some(rhs)) => {
                let mut ones = 0;
                for (x, y) in lhs.as_slice_mut().iter_mut().zip(rhs.as_slice()) {
                    *x |= *y;
                    ones += x.count1();
                }
                self.ones = ucast(ones);
            }
            (None, Some(rhs)) => {
                let mut dst = A::empty();
                dst.as_slice_mut().copy_from_slice(rhs.as_slice());
                self.data = Some(Box::new(dst));
            }
            _ => {}
        }
    }
}

impl<A: BlockArray> BitXor<&'_ Block<A>> for &'_ Block<A> {
    type Output = Block<A>;

    fn bitxor(self, that: &Block<A>) -> Self::Output {
        match (self.data.as_ref(), that.data.as_ref()) {
            (Some(lhs), Some(rhs)) => {
                let mut data = A::empty();
                let mut ones = 0;
                let slice = lhs
                    .as_slice()
                    .iter()
                    .zip(rhs.as_slice())
                    .map(|(&a, &b)| a ^ b);

                for (x, y) in data.as_slice_mut().iter_mut().zip(slice) {
                    *x = y;
                    ones += x.count1();
                }
                Block {
                    ones: ucast(ones),
                    data: Some(Box::new(data)),
                }
            }
            (Some(lhs), None) => Block {
                ones: self.ones,
                data: Some(lhs.clone()),
            },
            (None, Some(rhs)) => Block {
                ones: self.ones,
                data: Some(rhs.clone()),
            },
            _ => Block::empty(),
        }
    }
}

impl<A: BlockArray> BitXorAssign<&'_ Block<A>> for Block<A> {
    fn bitxor_assign(&mut self, that: &Block<A>) {
        match (self.data.as_mut(), that.data.as_ref()) {
            (Some(lhs), Some(rhs)) => {
                let mut ones = 0;
                for (x, y) in lhs.as_slice_mut().iter_mut().zip(rhs.as_slice()) {
                    *x ^= *y;
                    ones += x.count1();
                }
                self.ones = ucast(ones);
            }
            (None, Some(rhs)) => {
                let mut dst = A::empty();
                dst.as_slice_mut().copy_from_slice(rhs.as_slice());
                self.data = Some(Box::new(dst));
            }
            _ => {}
        }
    }
}

impl<A: BlockArray> Not for Block<A> {
    type Output = Block<A>;
    fn not(self) -> Self::Output {
        match self.data {
            Some(mut arr) => {
                let ones = {
                    let mut acc = 0;
                    for v in arr.as_slice_mut().iter_mut() {
                        *v = !*v;
                        acc += v.count1();
                    }
                    acc
                };
                Block {
                    ones: ucast(ones),
                    data: if ones > 0 { Some(arr) } else { None },
                }
            }
            None => Self::splat(!A::Value::ZERO),
        }
    }
}

impl<A: BlockArray> Not for &'_ Block<A> {
    type Output = Block<A>;
    fn not(self) -> Self::Output {
        match self.data {
            Some(ref arr) => {
                let mut out = A::splat(A::Value::ZERO);
                let mut acc = 0;
                for (a, b) in out.as_slice_mut().iter_mut().zip(arr.as_slice()) {
                    *a = !*b;
                    acc += a.count1();
                }
                Block {
                    ones: ucast(acc),
                    data: if acc > 0 { Some(Box::new(out)) } else { None },
                }
            }
            None => Block::splat(!A::Value::ZERO),
        }
    }
}

// FIXME: Revisit here when const generics is stabilized.

/// `[T; N]` is almost same with `[T]` where T is an UnsignedInt,
/// except that `[T; N]` implements `FiniteBlocks`.
macro_rules! implBlockArray {
    ($( ($Val:ty, $LEN:expr) ),*) => ($(
        impl crate::private::Sealed for [$Val; $LEN] {}
        impl BlockArray for [$Val; $LEN] {
            type Value = $Val;

            const LEN: usize = $LEN;

            fn splat(value: Self::Value) -> Self { [value; $LEN] }

            fn as_slice(&self) -> &[$Val] { &self[..] }

            fn as_slice_mut(&mut self) -> &mut [$Val] { &mut self[..] }
        }

        impl From<Block<Self>> for [$Val; $LEN] {
            fn from(block: Block<Self>) -> Self {
                block.data.map_or_else(Self::empty, |boxed| *boxed)
            }
        }
        impl From<&'_ Block<Self>> for [$Val; $LEN] {
            fn from(block: &Block<Self>) -> Self {
                block.data.as_ref().map_or_else(Self::empty, |boxed| {
                    let mut array = Self::splat(0);
                    array.copy_from_slice(&boxed[..]);
                    array
                })
            }
        }

        impl FiniteBits for [$Val; $LEN] {
            const BITS: u64 = <$Val as FiniteBits>::BITS * $LEN as u64;
            fn empty() -> Self {
                [0; $LEN]
            }
        }

        impl Count for [$Val; $LEN] {
            fn bits(&self) -> u64 {
                Self::BITS
            }
            fn count1(&self) -> u64 {
                self.as_ref().count1()
            }
        }

        impl Access for [$Val; $LEN] {
            fn access(&self, i: u64) -> bool {
                assert!(i < self.bits(), bit::OUT_OF_BOUNDS);
                self.as_ref().access(i)
            }
            fn iterate<'a>(&'a self) -> Box<dyn Iterator<Item = u64> + 'a> {
                self.as_ref().iterate()
            }
        }

        impl Rank for [$Val; $LEN] {
            fn rank1(&self, i: u64) -> u64 {
                assert!(i <= self.bits(), bit::OUT_OF_BOUNDS);
                self.as_ref().rank1(i)
            }
        }

        impl Select1 for [$Val; $LEN] {
            fn select1(&self, n: u64) -> Option<u64> {
                self.as_ref().select1(n)
            }
        }
        impl Select0 for [$Val; $LEN] {
            fn select0(&self, n: u64) -> Option<u64> {
                self.as_ref().select0(n)
            }
        }

        impl Assign<u64> for [$Val; $LEN] {
            type Output = <[$Val] as Assign<u64>>::Output;

            fn set1(&mut self, i: u64) -> Self::Output {
                assert!(i < self.bits(), bit::OUT_OF_BOUNDS);
                self.as_mut().set1(i)
            }
            fn set0(&mut self, i: u64) -> Self::Output {
                assert!(i < self.bits(), bit::OUT_OF_BOUNDS);
                self.as_mut().set0(i)
            }
        }

        impl Assign<Range<u64>> for [$Val; $LEN] {
            type Output = <[$Val] as Assign<Range<u64>>>::Output;

            fn set1(&mut self, i: Range<u64>) -> Self::Output {
                self.as_mut().set1(i)
            }
            fn set0(&mut self, i: Range<u64>) -> Self::Output {
                self.as_mut().set0(i)
            }
        }
    )*)
}
#[rustfmt::skip]
implBlockArray!(
    (u8,   8192usize),
    (u16,  4096usize),
    (u32,  2048usize),
    (u64,  1024usize),
    (u128, 512usize)
);

#[cfg(target_pointer_width = "32")]
implBlockArray!((usize, 2048usize));
#[cfg(target_pointer_width = "64")]
implBlockArray!((usize, 1024usize));

impl<T> bit::Map<T> {
    fn access<U: ?Sized>(data: &U, i: u64) -> bool
    where
        T: FiniteBits + Access,
        U: AsRef<[T]> + Count,
    {
        assert!(i < data.bits(), bit::OUT_OF_BOUNDS);
        let (i, o) = bit::divmod::<usize>(i, T::BITS);
        data.as_ref().get(i).map_or(false, |t| t.access(o))
    }

    fn rank1<U: ?Sized>(data: &U, i: u64) -> u64
    where
        T: FiniteBits + Rank,
        U: AsRef<[T]> + Count,
    {
        assert!(i <= data.bits(), bit::OUT_OF_BOUNDS);
        let (i, o) = bit::divmod(i, T::BITS);
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

impl<T> Count for bit::Map<T>
where
    T: FiniteBits,
{
    /// # Examples
    ///
    /// ```
    /// use compacts::bit::{Map, ops::Count};
    /// let map = Map::with([0u64, 0b10101100000, 0b0000100000]);
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

impl<T> Access for bit::Map<T>
where
    T: FiniteBits + Access,
{
    /// Test bit at a given position.
    ///
    /// # Examples
    ///
    /// ```
    /// use compacts::bit::{Map, ops::Access};
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
    /// # use compacts::bit::{Map, ops::Access};
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
    /// # use compacts::bit::{Map, ops::Access};
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
        bit::Map::access(self, i)
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
        bit::Map::access(self, i)
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
                    let offset = ucast::<usize, u64>(i) * T::BITS;
                    t.iterate().map(move |j| j + offset)
                }),
        )
    }
}

impl<T> Rank for bit::Map<T>
where
    T: FiniteBits + Rank,
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
        bit::Map::rank1(self, i)
    }
}

impl<T> Rank for [T]
where
    T: FiniteBits + Rank,
{
    fn rank1(&self, i: u64) -> u64 {
        bit::Map::rank1(self, i)
    }
}

impl<T> Select1 for bit::Map<T>
where
    T: FiniteBits + Select1,
{
    /// # Examples
    ///
    /// ```
    /// use compacts::bit::{Map, ops::Select1};
    /// let map = Map::with([0b_00000000_u8, 0b_01000000, 0b_00001001]);
    /// assert_eq!(map.select1(0), Some(14));
    /// assert_eq!(map.select1(1), Some(16));
    /// assert_eq!(map.select1(2), Some(19));
    /// assert_eq!(map.select1(3), None);
    /// ```
    ///
    /// ```
    /// # use compacts::bit::{Map, ops::Select1};
    /// # let map = Map::with([0b_00000000_u8, 0b_01000000, 0b_00001001]);
    /// assert_eq!(map.as_ref().select1(0), Some(14));
    /// assert_eq!(map.as_ref().select1(1), Some(16));
    /// assert_eq!(map.as_ref().select1(2), Some(19));
    /// assert_eq!(map.as_ref().select1(3), None);
    /// ```
    fn select1(&self, n: u64) -> Option<u64> {
        bit::Map::select1(self, n)
    }
}

impl<T> Select1 for [T]
where
    T: FiniteBits + Select1,
{
    fn select1(&self, n: u64) -> Option<u64> {
        bit::Map::select1(self, n)
    }
}

impl<T> Select0 for bit::Map<T>
where
    T: FiniteBits + Select0,
{
    /// # Examples
    ///
    /// ```
    /// use compacts::bit::{Map, ops::Select0};
    /// let map = Map::with([0b_11110111_u8, 0b_11111110, 0b_10010011]);
    /// assert_eq!(map.select0(0), Some(3));
    /// assert_eq!(map.select0(1), Some(8));
    /// assert_eq!(map.select0(2), Some(18));
    /// assert_eq!(map.select0(6), Some(24));
    /// ```
    /// ```
    /// # use compacts::bit::{Map, ops::Select0};
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

impl<T> Assign<u64> for bit::Map<T>
where
    T: FiniteBits + Access + Assign<u64>,
{
    type Output = ();

    /// Enable bit at a given position.
    ///
    /// # Examples
    ///
    /// ```
    /// use compacts::bit::{Map, Block, ops::{Access, Assign}};
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
        assert!(i < self.bits(), bit::OUT_OF_BOUNDS);
        let (i, o) = bit::divmod::<usize>(i, T::BITS);

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
        let (i, o) = bit::divmod::<usize>(i, T::BITS);

        if i < self.data.len() && self.data[i].access(o) {
            self.ones -= 1;
            self.data[i].set0(o);
        }
    }

    // fn flip(&mut self, i: u64) -> Self::Output {
    //     assert!(i < self.bits(), bit::OUT_OF_BOUNDS);
    //     let (i, o) = bit::divmod::<usize>(i, T::BITS);
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
        assert!(i < self.bits(), bit::OUT_OF_BOUNDS);
        let (i, o) = bit::divmod::<usize>(i, T::BITS);
        self[i].set1(o)
    }

    fn set0(&mut self, i: u64) -> Self::Output {
        assert!(i < self.bits(), bit::OUT_OF_BOUNDS);
        let (i, o) = bit::divmod::<usize>(i, T::BITS);
        self[i].set0(o)
    }

    // fn flip(&mut self, i: u64) -> Self::Output {
    //     assert!(i < self.bits(), bit::OUT_OF_BOUNDS);
    //     let (i, o) = bit::divmod::<usize>(i, T::BITS);
    //     self[i].flip(o)
    // }
}

impl<T> Assign<Range<u64>> for bit::Map<T>
where
    T: FiniteBits + Assign<Range<u64>, Output = u64>,
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

                let (head_index, head_offset) = bit::divmod::<usize>(i, T::BITS);
                let (last_index, last_offset) = bit::divmod::<usize>(j, T::BITS);
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

                let (head_index, head_offset) = bit::divmod::<usize>(i, T::BITS);
                let (last_index, last_offset) = bit::divmod::<usize>(j, T::BITS);
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

                let (head_index, head_offset) = bit::divmod::<usize>(i, T::BITS);
                let (last_index, last_offset) = bit::divmod::<usize>(j, T::BITS);

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

impl<'a, V, U> std::iter::FromIterator<Cow<'a, V>> for bit::Map<U>
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

impl<'a, K, V, U> std::iter::FromIterator<bit::Entry<K, Cow<'a, V>>> for bit::KeyMap<K, U>
where
    K: UnsignedInt,
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

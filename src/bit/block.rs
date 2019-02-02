use std::{
    fmt,
    ops::{self, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Range},
};

use crate::bit::{self, cast, from_any_bounds, ops::*, Uint};

#[derive(Clone)]
pub struct Block<A: BlockArray> {
    ones: u32,
    data: Option<Box<A>>,
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
impl<A: BlockArray> Eq for Block<A> {}

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
    + Read<u8>
    + Read<u16>
    + Read<u32>
    + Read<u64>
    + Read<u128>
    + Read<usize>
{
    type Value: Uint;
    const LEN: usize;

    fn splat(value: Self::Value) -> Self;

    fn as_slice(&self) -> &[Self::Value];

    fn as_slice_mut(&mut self) -> &mut [Self::Value];
}

impl<A: BlockArray> From<A> for Block<A> {
    fn from(array: A) -> Self {
        let ones = cast(array.count1());
        let data = Some(Box::new(array));
        Block { ones, data }
    }
}
impl<A: BlockArray> From<&'_ A> for Block<A> {
    fn from(array: &A) -> Self {
        let ones = cast(array.count1());
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
        let ones = cast::<u64, u32>(value.count1()) * cast::<usize, u32>(A::LEN);
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
        self.ones = cast(this.count1());
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
        self.ones += cast::<u64, u32>(out);
        out
    }
    fn set0(&mut self, i: Range<u64>) -> Self::Output {
        if let Some(arr) = self.data.as_mut() {
            let out = arr.set0(i);
            self.ones -= cast::<u64, u32>(out);
            out
        } else {
            0u64
        }
    }
}

impl<W: Uint, A: BlockArray + Read<W>> Read<W> for Block<A> {
    /// # Examples
    ///
    /// ```
    /// use compacts::bit::{Block, ops::Read, ops::FiniteBits};
    ///
    /// let block = Block::<[u8; 8192]>::empty();
    /// assert_eq!(Read::<u64>::read(&block, 100..163), 0);
    /// assert_eq!(Read::<u64>::read(&block, 163..180), 0);
    ///
    /// let block = Block::<[u8; 8192]>::splat(0b_0001_1100);
    /// assert_eq!(Read::<u8>::read(&block, 0..3),  0b_0000_0100_u8);
    /// assert_eq!(Read::<u8>::read(&block, 0..4),  0b_0000_1100_u8);
    /// assert_eq!(Read::<u8>::read(&block, 6..12), 0b_0011_0000_u8);
    /// ```
    fn read<R: std::ops::RangeBounds<u64>>(&self, r: R) -> W {
        if let Some(arr) = self.data.as_ref() {
            arr.read(r)
        } else {
            let r = from_any_bounds(&r, self.bits());
            assert!(r.start < r.end && r.end - r.start <= W::BITS);
            W::ZERO
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
                    ones: cast(ones),
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
                self.ones = cast(ones);
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
                    ones: cast(ones),
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
                self.ones = cast(ones);
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
                    ones: cast(ones),
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
                self.ones = cast(ones);
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
                    ones: cast(ones),
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
                    ones: cast(acc),
                    data: if acc > 0 { Some(Box::new(out)) } else { None },
                }
            }
            None => Block::splat(!A::Value::ZERO),
        }
    }
}

// FIXME: Revisit here when const generics is stabilized.

/// `[T; N]` is almost same with `[T]` where T is an Uint,
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

        impl<W: Uint> Read<W> for [$Val; $LEN] {
            fn read<R: std::ops::RangeBounds<u64>>(&self, r: R) -> W {
                self.as_ref().read(r)
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

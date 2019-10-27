//! Module `fixed_bits` provides primitives to interact with fixed size bits.

use std::{
    borrow::{Borrow, BorrowMut, ToOwned},
    fmt::{self, Debug, Formatter},
    hash::{Hash, Hasher},
    mem,
    ops::{Deref, DerefMut},
    slice,
};

use crate::{ops::private::Sealed, ops::*};

use super::{Word, Words};

/// A finite, fixed size bit slice.
#[repr(transparent)]
pub struct FixedBits<T: Words>(
    // unsized slice but `len()` should be always equals to T::LEN
    pub(crate) [T::Word],
);

impl<T: Words> Debug for FixedBits<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: Words> PartialEq for FixedBits<T> {
    fn eq(&self, that: &Self) -> bool {
        self.0 == that.0
    }
}
impl<T: Words> Eq for FixedBits<T> {}

impl<T: Words> Hash for FixedBits<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.len().hash(state);
        Hash::hash_slice(&self.0, state)
    }
}

impl<T: Words> From<Box<[T::Word]>> for Box<FixedBits<T>> {
    fn from(mut boxed: Box<[T::Word]>) -> Self {
        assert_eq!(boxed.as_ref().len(), T::SIZE);
        mem::transmute(boxed)
        // let output: Box<FixedBits<T>> = unsafe {
        //     let bits = FixedBits::<T>::from_raw_parts_mut(boxed.as_mut_ptr(), boxed.as_ref().len());
        //     Box::from_raw(bits)
        // };
        // mem::forget(boxed);
        // output
    }
}

impl<T: Words> Borrow<[T::Word]> for FixedBits<T> {
    #[inline]
    fn borrow(&self) -> &[T::Word] {
        &self.0
    }
}
impl<T: Words> BorrowMut<[T::Word]> for FixedBits<T> {
    #[inline]
    fn borrow_mut(&mut self) -> &mut [T::Word] {
        &mut self.0
    }
}

impl<T: Words> AsRef<[T::Word]> for FixedBits<T> {
    #[inline]
    fn as_ref(&self) -> &[T::Word] {
        &self.0
    }
}
impl<T: Words> AsMut<[T::Word]> for FixedBits<T> {
    #[inline]
    fn as_mut(&mut self) -> &mut [T::Word] {
        &mut self.0
    }
}

impl<'a, T: Words> IntoIterator for &'a FixedBits<T> {
    type Item = &'a T::Word;
    type IntoIter = slice::Iter<'a, T::Word>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl<'a, T: Words> IntoIterator for &'a mut FixedBits<T> {
    type Item = &'a mut T::Word;
    type IntoIter = slice::IterMut<'a, T::Word>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T: Words> FixedBits<T> {
    pub(crate) const ARRAY_BITS: u64 = T::SIZE as u64 * T::Word::BITS;

    /// Returns an empty instance.
    pub fn empty() -> Box<Self> {
        Box::<Self>::from(T::splat(T::Word::NONE).boxed())
    }

    /// Repeats `value` and build a boxed instance.
    pub fn splat(value: T::Word) -> Box<Self> {
        Box::<Self>::from(T::splat(value).boxed())
    }

    pub fn make<B: ?Sized + AsRef<[T::Word]>>(data: &B) -> &FixedBits<T> {
        let data = data.as_ref();
        assert_eq!(data.len(), T::SIZE);
        Self::make_unchecked(data)
    }

    pub fn make_mut<B: ?Sized + AsMut<[T::Word]>>(data: &mut B) -> &mut FixedBits<T> {
        let data = data.as_mut();
        assert_eq!(data.len(), T::SIZE);
        Self::make_unchecked_mut(data)
    }

    pub fn owned<B: ?Sized + AsRef<[T::Word]>>(data: &B) -> Box<FixedBits<T>> {
        let mut owned: Box<FixedBits<T>> = T::splat(T::Word::NONE).boxed().into();
        owned.copy_from_slice(data.as_ref());
        owned
    }

    fn make_unchecked(data: &[T::Word]) -> &FixedBits<T> {
        unsafe { &*(data as *const [T::Word] as *const FixedBits<T>) }
    }

    fn make_unchecked_mut(data: &mut [T::Word]) -> &mut FixedBits<T> {
        unsafe { &mut *(data as *mut [T::Word] as *mut FixedBits<T>) }
    }

    /// The total size of the std::slice must be no larger than isize::MAX bytes in memory.
    pub unsafe fn from_raw_parts<'a>(data: *const T::Word, len: usize) -> &'a FixedBits<T> {
        assert_eq!(len, T::SIZE);
        FixedBits::make(std::slice::from_raw_parts(data, len))
    }

    /// The total size of the std::slice must be no larger than isize::MAX bytes in memory.
    pub unsafe fn from_raw_parts_mut<'a>(data: *mut T::Word, len: usize) -> &'a mut FixedBits<T> {
        assert_eq!(len, T::SIZE);
        FixedBits::make_mut(std::slice::from_raw_parts_mut(data, len))
    }

    #[inline]
    pub fn as_bits(&self) -> &FixedBits<T::Word> {
        FixedBits::make(&self.0)
    }
    #[inline]
    pub fn as_mut_bits(&mut self) -> &mut FixedBits<T::Word> {
        FixedBits::make_mut(&mut self.0)
    }
}

impl<T: Words> FixedBits for Box<FixedBits<T>> {
    const SIZE: u64 = FixedBits::<T>::ARRAY_BITS;
    fn none() -> Self {
        FixedBits::<T>::empty()
    }
}

impl<T: Words> BitLen for FixedBits<T> {
    #[inline]
    fn size(&self) -> u64 {
        Self::ARRAY_BITS
    }

    #[inline]
    fn count1(&self) -> u64 {
        BitLen::count1(FixedBits::make(self))
    }
    #[inline]
    fn count0(&self) -> u64 {
        BitLen::count0(FixedBits::make(self))
    }

    #[inline]
    fn all(&self) -> bool {
        BitLen::all(FixedBits::make(self))
    }
    #[inline]
    fn any(&self) -> bool {
        BitLen::any(FixedBits::make(self))
    }
}

impl<T: Words> BitGet for FixedBits<T> {
    #[inline]
    fn get(&self, i: u64) -> bool {
        BitGet::get(FixedBits::make(self), i)
    }

    #[inline]
    fn getn<U: Word>(&self, i: u64, n: u64) -> U {
        BitGet::getn(FixedBits::make(self), i, n)
    }
}

impl<T: Words> BitsMut for FixedBits<T> {
    #[inline]
    fn put1(&mut self, i: u64) -> bool {
        BitsMut::put1(FixedBits::make_mut(self), i)
    }
    #[inline]
    fn put0(&mut self, i: u64) -> bool {
        BitsMut::put0(FixedBits::make_mut(self), i)
    }

    #[inline]
    fn putn<U: Word>(&mut self, i: u64, n: u64, w: U) {
        BitsMut::putn(FixedBits::make_mut(self), i, n, w)
    }
}

impl<T: Words> BitRank for FixedBits<T> {
    #[inline]
    fn rank1(&self, i: u64, j: u64) -> u64 {
        BitRank::rank1(FixedBits::make(self), i, j)
    }
    #[inline]
    fn rank0(&self, i: u64, j: u64) -> u64 {
        BitRank::rank1(FixedBits::make(self), i, j)
    }
}

impl<T: Words> BitSelect1 for FixedBits<T> {
    #[inline]
    fn select1(&self, n: u64) -> Option<u64> {
        BitSelect1::select1(FixedBits::make(self), n)
    }
}
impl<T: Words> BitSelect0 for FixedBits<T> {
    #[inline]
    fn select0(&self, n: u64) -> Option<u64> {
        BitSelect0::select0(FixedBits::make(self), n)
    }
}

impl<T: Words> mask::Intersection<FixedBits<T>> for FixedBits<T> {
    #[inline]
    fn intersection(&mut self, bits: &FixedBits<T>) {
        for (v1, &v2) in self.iter_mut().zip(bits) {
            *v1 &= v2;
        }
    }
}
impl<T: Words> mask::Union<FixedBits<T>> for FixedBits<T> {
    #[inline]
    fn union(&mut self, bits: &FixedBits<T>) {
        for (v1, &v2) in self.iter_mut().zip(bits) {
            *v1 |= v2;
        }
    }
}
impl<T: Words> mask::Difference<FixedBits<T>> for FixedBits<T> {
    #[inline]
    fn difference(&mut self, bits: &FixedBits<T>) {
        for (v1, &v2) in self.iter_mut().zip(bits) {
            *v1 &= !v2;
        }
    }
}
impl<T: Words> mask::SymmetricDifference<FixedBits<T>> for FixedBits<T> {
    #[inline]
    fn symmetric_difference(&mut self, bits: &FixedBits<T>) {
        for (v1, &v2) in self.iter_mut().zip(bits) {
            *v1 ^= v2;
        }
    }
}

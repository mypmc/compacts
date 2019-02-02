use std::{slice, vec};

use crate::{bits::cast, bits::Uint, bits::*};

use super::{Bin, Map, Run, BLOCK_SIZE, OUT_OF_BOUNDS};

impl<T> Default for Map<T> {
    fn default() -> Self {
        Map(None)
    }
}

impl<T: Uint> FiniteBits for Map<T> {
    const BITS: u64 = BLOCK_SIZE as u64;
    fn empty() -> Self {
        Self::default()
    }
}

impl<T: Uint> Count for Map<T> {
    fn bits(&self) -> u64 {
        Self::BITS
    }
    fn count1(&self) -> u64 {
        self.as_ref().map_or(0, |xs| xs.count1())
    }
}

impl<T: Uint> Access for Map<T> {
    fn access(&self, i: u64) -> bool {
        assert!(i < Self::BITS, OUT_OF_BOUNDS);
        self.as_ref().map_or(false, |slice| slice.access(i))
    }
}

impl<T: Uint> Assign<u64> for Map<T> {
    type Output = <[T] as Assign<u64>>::Output;

    fn set1(&mut self, i: u64) -> Self::Output {
        assert!(i < Self::BITS, OUT_OF_BOUNDS);
        self.alloc().set1(i)
    }

    fn set0(&mut self, i: u64) -> Self::Output {
        assert!(i < Self::BITS, OUT_OF_BOUNDS);
        self.alloc().set0(i)
    }
}

impl<T: Uint> Assign<std::ops::Range<u64>> for Map<T>
where
    [T]: Assign<std::ops::Range<u64>>,
{
    // type Output = <[T] as Assign<std::ops::Range<u64>>>::Output;
    type Output = ();

    fn set1(&mut self, i: std::ops::Range<u64>) -> Self::Output {
        Assign::set1(self.alloc(), i);
    }
    fn set0(&mut self, i: std::ops::Range<u64>) -> Self::Output {
        Assign::set0(self.alloc(), i);
    }
}

impl<T: Uint> Rank for Map<T> {
    fn rank1(&self, i: u64) -> u64 {
        assert!(i <= Self::BITS, OUT_OF_BOUNDS);
        self.as_ref().map_or(0, |cow| cow.rank1(i))
    }
}

impl<T: Uint> Select1 for Map<T> {
    fn select1(&self, n: u64) -> Option<u64> {
        self.as_ref().and_then(|xs| xs.select1(n))
    }
}
impl<T: Uint> Select0 for Map<T> {
    fn select0(&self, n: u64) -> Option<u64> {
        self.as_ref().map_or(Some(n), |bv| bv.select0(n))
    }
}

impl<T> Map<T> {
    pub(crate) fn as_ref(&self) -> Option<&[T]> {
        self.0.as_ref().map(|b| b.as_ref())
    }

    pub(crate) fn as_mut(&mut self) -> Option<&mut [T]> {
        self.0.as_mut().map(|b| b.as_mut())
    }

    pub(crate) fn into_inner(self) -> Option<Box<[T]>> {
        self.0
    }
}

impl<T: Uint> Map<T> {
    pub(crate) const LEN: usize = (Self::BITS / T::BITS) as usize;
}

impl<T: Uint> From<Vec<T>> for Map<T> {
    fn from(mut vec: Vec<T>) -> Self {
        vec.resize(Self::LEN, T::ZERO);
        Map(Some(vec.into_boxed_slice()))
    }
}

impl<T: Uint> From<&'_ Bin> for Map<T> {
    fn from(bin: &Bin) -> Self {
        let mut vec = Map::<T>::splat(T::ZERO);
        for &i in bin.0.iter() {
            vec.set1(cast::<u16, u64>(i));
        }
        vec
    }
}

impl From<&'_ Run> for Map<u64> {
    fn from(run: &Run) -> Self {
        let mut vec = vec![0; Self::LEN];
        for range in run.0.iter() {
            let i = u64::from(*range.start());
            let len = u64::from(*range.end()) + 1 - i;
            vec.set1(i..i + len);
        }
        Self::from(vec)
    }
}

impl<T: Uint> Map<T> {
    /// Return an empty instance.
    pub fn empty() -> Self {
        Map(None)
    }

    /// Constructs a new instance with each element initialized to value.
    pub fn splat(value: T) -> Self {
        Map(Some(vec![value; Self::LEN].into_boxed_slice()))
    }

    /// Length of T, not bit size.
    pub fn len(&self) -> usize {
        match self.as_ref() {
            None => 0,
            Some(ref vec) => vec.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    // prepare to insert/remove.
    fn alloc(&mut self) -> &mut [T] {
        if self.0.is_none() {
            *self = Self::splat(T::ZERO);
        }
        self.as_mut().expect("unreachable")
    }

    pub fn iter<'r>(&'r self) -> impl Iterator<Item = T> + 'r {
        self.into_iter()
    }
}

pub struct Iter<'a, T: Uint>(Option<slice::Iter<'a, T>>);
pub struct IntoIter<T: Uint>(Option<vec::IntoIter<T>>);

impl<'r, T: Uint> IntoIterator for &'r Map<T> {
    type Item = T;
    type IntoIter = Iter<'r, T>;
    fn into_iter(self) -> Self::IntoIter {
        Iter(self.as_ref().map(|b| b.iter()))
    }
}

impl<T: Uint> IntoIterator for Map<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.0.map(|b| b.into_vec().into_iter()))
    }
}

impl<'a, T: Uint> Iterator for Iter<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.as_mut().and_then(|i| i.next().cloned())
    }
}
impl<T: Uint> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.as_mut().and_then(|i| i.next())
    }
}

impl<'b, T: Uint> std::ops::BitAndAssign<&'b Map<T>> for Map<T> {
    fn bitand_assign(&mut self, that: &'b Map<T>) {
        match (self.as_mut(), that.as_ref()) {
            (Some(lhs), Some(rhs)) => {
                assert_eq!(lhs.len(), rhs.len());
                for (x, y) in lhs.iter_mut().zip(rhs.iter()) {
                    *x &= *y;
                }
            }
            _ => {
                self.0 = None;
            }
        }
    }
}

impl<'b, T: Uint> std::ops::BitOrAssign<&'b Map<T>> for Map<T> {
    fn bitor_assign(&mut self, that: &'b Map<T>) {
        match (self.as_mut(), that.as_ref()) {
            (None, Some(buf)) => {
                let mut dst = vec![T::ZERO; buf.len()];
                dst.copy_from_slice(&buf[..]);
                self.0 = Some(dst.into_boxed_slice());
            }
            (Some(lhs), Some(rhs)) => {
                assert_eq!(lhs.len(), rhs.len());
                for (x, y) in lhs.iter_mut().zip(rhs.iter()) {
                    *x |= *y;
                }
            }
            _ => {}
        }
    }
}

impl<'b, T: Uint> std::ops::BitXorAssign<&'b Map<T>> for Map<T> {
    fn bitxor_assign(&mut self, that: &'b Map<T>) {
        match (self.as_mut(), that.as_ref()) {
            (None, Some(buf)) => {
                let mut dst = vec![T::ZERO; buf.len()];
                dst.copy_from_slice(&buf[..]);
                self.0 = Some(dst.into_boxed_slice());
            }
            (Some(lhs), Some(rhs)) => {
                assert_eq!(lhs.len(), rhs.len());
                for (x, y) in lhs.iter_mut().zip(rhs.iter()) {
                    *x ^= *y;
                }
            }
            _ => {}
        }
    }
}

impl<T: Uint> std::ops::Not for &'_ Map<T> {
    type Output = Map<T>;
    fn not(self) -> Self::Output {
        match self.as_ref() {
            Some(arr) => {
                let mut vec = arr.to_vec();
                let ones = {
                    let mut acc = 0;
                    for v in vec.iter_mut() {
                        *v = !*v;
                        acc += v.count1();
                    }
                    acc
                };
                Map(if ones > 0 {
                    Some(vec.into_boxed_slice())
                } else {
                    None
                })
            }
            None => Map::splat(!T::ZERO),
        }
    }
}

impl<T: Uint> std::ops::Not for Map<T> {
    type Output = Map<T>;
    fn not(self) -> Self::Output {
        match self.into_inner() {
            Some(mut vec) => {
                let ones = {
                    let mut acc = 0;
                    for v in vec.iter_mut() {
                        *v = !*v;
                        acc += v.count1();
                    }
                    acc
                };
                Map(if ones > 0 { Some(vec) } else { None })
            }
            None => Map::splat(!T::ZERO),
        }
    }
}

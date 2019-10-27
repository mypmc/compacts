use std::fmt;

use crate::{
    bits::{Word, Words},
    ops::*,
};

/// `Bin` has the fixed bit size specified by type parameter `T`.
#[derive(Clone)]
pub struct Bin<T: Words>(pub(crate) Option<Box<T>>);

impl<T: Words> fmt::Debug for Bin<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl<T: Words> PartialEq for Bin<T> {
    #[inline]
    fn eq(&self, that: &Self) -> bool {
        self.as_ref() == that.as_ref()
    }
}
impl<T: Words> Eq for Bin<T> {}

impl<T: Words> Bin<T> {
    /// Exposes a shared reference of an internal bit slice.
    #[inline(always)]
    pub fn as_ref(&self) -> Option<&[T::Word]> {
        self.0.as_ref().map(|ws| ws.as_ref_words())
    }

    /// Exposes a mutable reference of an internal bit slice.
    #[inline(always)]
    pub fn as_mut(&mut self) -> Option<&mut [T::Word]> {
        self.0.as_mut().map(|ws| ws.as_mut_words())
    }
}

impl<T: Words> From<T> for Bin<T> {
    #[inline]
    fn from(words: T) -> Bin<T> {
        Bin(Some(Box::new(words)))
    }
}

impl<T: Words> From<Option<T>> for Bin<T> {
    #[inline]
    fn from(words: Option<T>) -> Bin<T> {
        Bin(words.map(Box::new))
    }
}

// impl<T: Words> From<Box<[T::Word]>> for Bin<T> {
//     #[inline]
//     fn from(words: Box<[T::Word]>) -> Bin<T> {
//         Bin(Some(words))
//     }
// }

// impl<T: Words> From<Option<Box<[T::Word]>>> for Bin<T> {
//     #[inline]
//     fn from(words: Option<Box<[T::Word]>>) -> Bin<T> {
//         Bin(words)
//     }
// }

impl<T: Words> FixedBits for Bin<T> {
    const SIZE: usize = T::BITS;
    #[inline]
    fn none() -> Self {
        Bin(None)
    }
}

impl<T: Words> Bits for Bin<T> {
    #[inline]
    fn size(&self) -> usize {
        T::BITS
    }

    #[inline]
    fn count1(&self) -> usize {
        self.0.count1()
    }
    #[inline]
    fn count0(&self) -> usize {
        self.0.count0()
    }

    #[inline]
    fn all(&self) -> bool {
        self.0.all()
    }
    #[inline]
    fn any(&self) -> bool {
        self.0.any()
    }

    #[inline]
    fn bit(&self, i: usize) -> bool {
        self.0.bit(i)
    }

    #[inline]
    fn getn<W: Word>(&self, i: usize, n: usize) -> W {
        self.0.getn(i, n)
    }
}

impl<T: Words> BitRank for Bin<T> {
    #[inline]
    fn rank1(&self, i: usize, j: usize) -> usize {
        self.0.rank1(i, j)
    }
    #[inline]
    fn rank0(&self, i: usize, j: usize) -> usize {
        self.0.rank0(i, j)
    }
}

impl<T: Words> BitSelect for Bin<T> {
    #[inline]
    fn select1(&self, n: usize) -> Option<usize> {
        self.0.select1(n)
    }
    #[inline]
    fn select0(&self, n: usize) -> Option<usize> {
        self.0.select0(n)
    }
}

impl<T: Words> BitsMut for Bin<T> {
    #[inline]
    fn put1(&mut self, i: usize) -> &mut Self {
        self.0.put1(i);
        self
    }
    #[inline]
    fn put0(&mut self, i: usize) -> &mut Self {
        self.0.put0(i);
        self
    }
    #[inline]
    fn flip(&mut self, i: usize) -> &mut Self {
        self.0.flip(i);
        self
    }

    #[inline]
    fn putn<W: Word>(&mut self, i: usize, n: usize, w: W) {
        self.0.putn(i, n, w);
    }
}

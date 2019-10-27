use std::{
    iter::{FusedIterator, Zip},
    marker::PhantomData,
    ops::RangeBounds,
    slice,
};

use crate::{
    bits::{self, to_exclusive, Words},
    num::Word,
    ops::*,
    BitArray, BitMap,
};

mod search;
mod trace;

pub use {
    search::{Max, Min, Search, Top},
    trace::Counts,
};

use super::View;

/// `WaveletMatrix`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaveletMatrix<T, B> {
    // length of the original sequence
    size: usize,

    // `tips` and `fids` have same size, T::DEPTH

    // fully indexable dictionaries.
    fids: Vec<B>,

    // The boundary indices between bin0 and bin1.
    // This index is also equals to the number of zeros.
    tips: Vec<usize>,

    // symbol type
    _sym: PhantomData<T>,
}

impl<'a, T, B> From<&'a mut [T]> for WaveletMatrix<T, BitArray<B>>
where
    T: Code,
    B: FixedBits,
    BitArray<B>: From<Vec<B>>,
{
    /// Builds WaveletMatrix and returns the sorted symbols.
    /// Sorting is performed bit by bit so that symbols are sorted lexicographically.
    fn from(bin0: &'a mut [T]) -> Self {
        let size = bin0.len();
        let _sym = PhantomData;

        let mut bin1 = bin0.to_vec();
        let mut tips = vec![0; T::DEPTH];
        let mut fids = Vec::with_capacity(T::DEPTH);

        // for depth in 0..T::DEPTH {
        for (depth, tip) in tips.iter_mut().enumerate().take(T::DEPTH) {
            let mut node = vec![B::none(); bits::blocks_by(size, B::SIZE)];

            let mut l = 0; // the number of 0 in fids[depth]
            let mut r = 0; // the number of 1 in fids[depth]
            for i in 0..size {
                if bin0[i].bit(T::DEPTH - depth - 1) {
                    node.put1(i);
                    bin1[r] = bin0[i];
                    r += 1;
                } else {
                    bin0[l] = bin0[i];
                    l += 1;
                }
            }

            *tip = l;
            // tips[depth] = l;
            fids.push(BitArray::from(node));
            debug_assert_eq!(l + r, size);
            bin0[l..].copy_from_slice(&bin1[..r]);
        }

        WaveletMatrix {
            size,
            tips,
            fids,
            _sym,
        }
    }
}

impl<'a, T: Code, B: Words> From<&'a mut [T]> for WaveletMatrix<T, BitMap<B>> {
    /// Builds WaveletMatrix and returns the sorted symbols.
    /// Sorting is performed bit by bit so that symbols are sorted lexicographically.
    fn from(bin0: &mut [T]) -> Self {
        let size = bin0.len();

        let mut bin1 = bin0.to_vec();
        let mut tips = vec![0; T::DEPTH];
        let mut fids = Vec::with_capacity(T::DEPTH);

        // for depth in 0..T::DEPTH {
        for (depth, tip) in tips.iter_mut().enumerate().take(T::DEPTH) {
            let mut node = BitMap::none(size);

            let mut l = 0; // the number of 0 in fids[depth]
            let mut r = 0; // the number of 1 in fids[depth]
            for i in 0..size {
                if bin0[i].bit(T::DEPTH - depth - 1) {
                    node.put1(i);
                    bin1[r] = bin0[i];
                    r += 1;
                } else {
                    bin0[l] = bin0[i];
                    l += 1;
                }
            }

            *tip = l;
            // tips[depth] = l;

            fids.push(node);
            debug_assert_eq!(l + r, size);
            bin0[l..].copy_from_slice(&bin1[..r]);
        }

        WaveletMatrix {
            _sym: PhantomData,
            size,
            tips,
            fids,
        }
    }
}

impl<T, B> WaveletMatrix<T, B> {
    pub fn view<R: RangeBounds<usize>>(&self, range: R) -> View<'_, Self> {
        View {
            idx: to_exclusive(&range, self.size),
            seq: self,
        }
    }

    #[inline]
    pub(crate) fn rows(&self) -> Rows<'_, B> {
        Rows {
            rows: self.fids.iter().zip(&self.tips),
        }
    }
}

impl<T: Word, B: Bits> WaveletMatrix<T, B> {
    /// ```
    /// use compacts::{BitArray, WaveletMatrix};
    /// let vec = vec![5u8, 4, 5, 5, 2, 1, 5, 6, 1, 3, 5, 0];
    /// let mut cloned = vec.clone();
    /// let wav = WaveletMatrix::<u8, BitArray<u64>>::from(cloned.as_mut_slice());
    ///
    /// for (i, &v) in vec.iter().enumerate() {
    ///     assert_eq!(wav.view(..).get(i), Some(v));
    /// }
    /// for (i, &v) in vec[5..].iter().enumerate() {
    ///     assert_eq!(wav.view(5..).get(i), Some(v));
    /// }
    /// ```
    #[inline]
    pub fn get(&self, i: usize) -> Option<T> {
        self.view(..).get(i)
    }

    #[inline]
    pub fn quantile(&self, k: usize) -> Option<(usize, T)> {
        self.view(..).quantile(k)
    }
}

impl<T: Word, B: Bits> WaveletMatrix<T, B> {
    #[inline]
    pub fn search<Min, Max>(&self, min: Min, max: Max) -> Search<'_, Self>
    where
        Min: Into<Option<T>>,
        Max: Into<Option<T>>,
    {
        self.view(..).search(min, max)
    }

    #[inline]
    pub fn top(&self) -> Top<'_, Self> {
        self.view(..).top()
    }

    #[inline]
    pub fn min(&self) -> Min<'_, Self> {
        self.view(..).min()
    }

    #[inline]
    pub fn max(&self) -> Max<'_, Self> {
        self.view(..).max()
    }

    #[inline]
    pub fn topk(&self, k: usize) -> Vec<(usize, T)> {
        self.view(..).topk(k)
    }

    #[inline]
    pub fn mink(&self, k: usize) -> Vec<(usize, T)> {
        self.view(..).mink(k)
    }

    #[inline]
    pub fn maxk(&self, k: usize) -> Vec<(usize, T)> {
        self.view(..).maxk(k)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Rows<'a, B> {
    rows: Zip<slice::Iter<'a, B>, slice::Iter<'a, usize>>,
}

#[derive(Debug, Clone)]
pub(crate) struct Row<'a, B> {
    fid: &'a B,
    tip: usize,
}

impl<'a, B> Iterator for Rows<'a, B> {
    type Item = Row<'a, B>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.rows.next().map(|(fid, &tip)| Row { fid, tip })
    }
}

impl<'a, B> DoubleEndedIterator for Rows<'a, B> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.rows.next_back().map(|(fid, &tip)| Row { fid, tip })
    }
}

impl<'a, B> ExactSizeIterator for Rows<'a, B> {
    #[inline]
    fn len(&self) -> usize {
        self.rows.len()
    }
}

impl<'a, B> FusedIterator for Rows<'a, B> {}

impl<T: Code, B: Bits> Text for WaveletMatrix<T, B> {
    type Code = T;

    /// Returns the number of elements in the sequence.
    #[inline]
    fn size(&self) -> usize {
        self.size
    }

    /// Counts the number of elements.
    #[inline]
    fn count(&self, val: &Self::Code) -> usize {
        self.view(..).counts(val).eq
    }
}

// impl<T, B> Rank<usize> for WaveletMatrix<T, B>
// where
//     T: Code,
//     B: Bits,
// {
//     #[inline]
//     fn rank(&self, val: &Self::Code, i: usize) -> usize {
//         self.view(0..i).counts(val).eq
//     }
// }

impl<T: Code, B: Bits, R: RangeBounds<usize>> Rank<R> for WaveletMatrix<T, B> {
    #[inline]
    fn rank(&self, sym: &Self::Code, range: R) -> usize {
        self.view(range).counts(sym).eq
    }
}

impl<T: Code, B: Bits> Select for WaveletMatrix<T, B> {
    #[inline]
    fn select(&self, sym: &Self::Code, n: usize) -> Option<usize> {
        self.view(..).select(sym, n)
    }
}

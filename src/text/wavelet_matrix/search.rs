use std::{cmp, collections::BinaryHeap, marker::PhantomData};

use crate::{
    num::Word,
    ops::{Bits, Code, Text},
};

use super::{View, WaveletMatrix};

/// `Search` is a builder of iterators that iterates over values
/// that satisfy `min <= value < max` in `[i, j)` .
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Search<'a, T: Text> {
    view: View<'a, T>,
    min: Option<T::Code>,
    max: Option<T::Code>,
}

/// Enumerates values from most frequent item.
pub struct Top<'a, T: Text>(Option<Heap<'a, cmpby::Top, T>>);
/// Enumerates values in ascending order.
pub struct Min<'a, T: Text>(Option<Heap<'a, cmpby::Min, T>>);
/// Enumerates values in descending order.
pub struct Max<'a, T: Text>(Option<Heap<'a, cmpby::Max, T>>);

#[derive(Debug, Clone)]
struct Heap<'a, By, T: Text> {
    seq: &'a T,
    min: Option<T::Code>,
    max: Option<T::Code>,
    bin: BinaryHeap<Probe<T::Code, By>>,
}

#[derive(Debug, Clone)]
struct Probe<T, U> {
    index: (usize, usize),
    depth: usize,
    value: T,
    _kind: PhantomData<U>,
}

impl<'a, T: Text> Search<'a, T>
where
    T::Code: Word,
{
    /// Enumerates value that satisfy `min <= value < max` in ascending order.
    pub fn min(self) -> Min<'a, T> {
        Min(self.heap())
    }

    /// Enumerates value that satisfy `min <= value < max` in descending order.
    pub fn max(self) -> Max<'a, T> {
        Max(self.heap())
    }

    /// Enumerates value that satisfy `min <= value < max` from most frequent one.
    pub fn top(self) -> Top<'a, T> {
        Top(self.heap())
    }

    fn heap<By>(self) -> Option<Heap<'a, By, T>>
    where
        Probe<T::Code, By>: Ord,
    {
        let min = self.min;
        let max = self.max;
        let seq = self.view.seq;
        self.view.idx.map(|index| Heap {
            seq,
            min,
            max,
            bin: BinaryHeap::from(vec![Probe {
                depth: 0,
                index: index,
                value: T::Code::MIN,
                _kind: PhantomData,
            }]),
        })
    }
}

impl<'a, T: Word, B: Bits> Search<'a, WaveletMatrix<T, B>> {
    /// Short for `top().take(k).collect()`.
    ///
    /// ```
    /// use compacts::{WaveletMatrix, BitArray};
    /// let mut vec = vec![5u8, 4, 5, 5, 2, 1, 5, 6, 1, 3, 5, 0];
    /// let wm = WaveletMatrix::<u8, BitArray<u64>>::from(vec.as_mut_slice());
    ///
    /// // [ 5, 4, 5, 5, 2, 1, 5, 6, 1, 3, 5, 0 ]
    /// let view = wm.view(..);
    ///
    /// assert_eq!(view.topk(1), vec![(5, 5)]);
    /// assert_eq!(view.topk(3), vec![(5, 5), (2, 1), (1, 6)]);
    ///
    /// assert_eq!(view.search(2, 5).topk(1), vec![(1, 4)]);
    /// assert_eq!(view.search(2, 5).topk(3), vec![(1, 4), (1, 3), (1, 2)]);
    /// assert_eq!(view.search(2, None).topk(1), vec![(5, 5)]);
    /// assert_eq!(view.search(2, None).topk(3), vec![(5, 5), (1, 6), (1, 4)]);
    ///
    /// // [ 5, 4, 5, [ 5, 2, 1, 5, 6, 1, 3, 5 ], 0 ]
    /// let view = wm.view(3..11);
    ///
    /// assert_eq!(view.topk(1), vec![(3, 5)]);
    /// assert_eq!(view.topk(3), vec![(3, 5), (2, 1), (1, 6)]);
    ///
    /// assert_eq!(view.search(3, 6).topk(1), vec![(3, 5)]);
    /// assert_eq!(view.search(3, 6).topk(3), vec![(3, 5), (1, 3)]);
    /// ```
    #[inline]
    pub fn topk(self, k: usize) -> Vec<(usize, T)> {
        self.top().take(k).collect::<Vec<_>>()
    }

    /// ```
    /// use compacts::{WaveletMatrix, BitArray};
    /// let mut vec = vec![5u8, 4, 5, 5, 2, 1, 5, 6, 1, 3, 5, 0];
    /// let wm = WaveletMatrix::<u8, BitArray<u64>>::from(vec.as_mut_slice());
    ///
    /// // [ 5, 4, 5, 5, 2, 1, 5, 6, 1, 3, 5, 0 ]
    /// assert_eq!(wm.view(..).mink(0), vec![]);
    /// assert_eq!(wm.view(..).mink(1), vec![(1, 0)]);
    /// assert_eq!(wm.view(..).mink(3), vec![(1, 0), (2, 1), (1, 2)]);
    /// assert_eq!(wm.view(..).mink(5), vec![(1, 0), (2, 1), (1, 2), (1, 3), (1, 4)]);
    ///
    /// // [ 5, 4, 5, [ 5, 2, 1, 5, 6, 1, 3, 5 ], 0 ]
    /// assert_eq!(wm.view(3..11).mink(0), vec![]);
    /// assert_eq!(wm.view(3..11).mink(1), vec![(2, 1)]);
    /// assert_eq!(wm.view(3..11).mink(3), vec![(2, 1), (1, 2), (1, 3)]);
    /// assert_eq!(wm.view(3..11).mink(5), vec![(2, 1), (1, 2), (1, 3), (3, 5), (1, 6)]);
    /// ```
    #[inline]
    pub fn mink(self, k: usize) -> Vec<(usize, T)> {
        self.min().take(k).collect::<Vec<_>>()
    }

    /// ```
    /// use compacts::{WaveletMatrix, BitArray};
    /// let mut vec = vec![5u8, 4, 5, 5, 2, 1, 5, 6, 1, 3, 5, 0];
    /// let wm = WaveletMatrix::<u8, BitArray<u64>>::from(vec.as_mut_slice());
    ///
    /// // [ 5, 4, 5, 5, 2, 1, 5, 6, 1, 3, 5, 0 ]
    /// assert_eq!(wm.view(..).maxk(0), vec![]);
    /// assert_eq!(wm.view(..).maxk(1), vec![(1, 6)]);
    /// assert_eq!(wm.view(..).maxk(3), vec![(1, 6), (5, 5), (1, 4)]);
    /// assert_eq!(wm.view(..).maxk(5), vec![(1, 6), (5, 5), (1, 4), (1, 3), (1, 2)]);
    ///
    /// // [ 5, 4, 5, [ 5, 2, 1, 5, 6, 1, 3, 5 ], 0 ]
    /// assert_eq!(wm.view(3..11).maxk(0), vec![]);
    /// assert_eq!(wm.view(3..11).maxk(1), vec![(1, 6)]);
    /// assert_eq!(wm.view(3..11).maxk(3), vec![(1, 6), (3, 5), (1, 3)]);
    /// assert_eq!(wm.view(3..11).maxk(5), vec![(1, 6), (3, 5), (1, 3), (1, 2), (2, 1)]);
    /// ```
    #[inline]
    pub fn maxk(self, k: usize) -> Vec<(usize, T)> {
        self.max().take(k).collect::<Vec<_>>()
    }
}

impl<'a, T: Text> View<'a, T>
where
    T::Code: Word,
{
    /// Builds a search such that return values satisfy `min <= value < max`.
    pub fn search<Min, Max>(&self, min: Min, max: Max) -> Search<'a, T>
    where
        Min: Into<Option<T::Code>>,
        Max: Into<Option<T::Code>>,
    {
        let min = min.into();
        let max = max.into();
        Search {
            view: View {
                idx: self.idx,
                seq: self.seq,
            },
            min,
            max,
        }
    }

    #[inline]
    pub fn top(&self) -> Top<'a, T> {
        self.search(None, None).top()
    }
    #[inline]
    pub fn min(&self) -> Min<'a, T> {
        self.search(None, None).min()
    }
    #[inline]
    pub fn max(&self) -> Max<'a, T> {
        self.search(None, None).max()
    }
}

impl<'a, T: Word, B: Bits> View<'a, WaveletMatrix<T, B>> {
    #[inline]
    pub fn topk(&self, k: usize) -> Vec<(usize, T)> {
        self.search(None, None).topk(k)
    }

    #[inline]
    pub fn mink(&self, k: usize) -> Vec<(usize, T)> {
        self.min().take(k).collect::<Vec<_>>()
    }

    #[inline]
    pub fn maxk(&self, k: usize) -> Vec<(usize, T)> {
        self.max().take(k).collect::<Vec<_>>()
    }
}

impl<'a, T: Word, B: Bits> Iterator for Top<'a, WaveletMatrix<T, B>> {
    type Item = (usize, T);
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.as_mut().and_then(|heap| heap.next())
    }
}

impl<'a, T: Word, B: Bits> Iterator for Min<'a, WaveletMatrix<T, B>> {
    type Item = (usize, T);
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.as_mut().and_then(|heap| heap.next())
    }
}

impl<'a, T: Word, B: Bits> Iterator for Max<'a, WaveletMatrix<T, B>> {
    type Item = (usize, T);
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.as_mut().and_then(|heap| heap.next())
    }
}

impl<'a, By, T: Text> Heap<'a, By, T>
where
    T::Code: Word,
    Probe<T::Code, By>: Ord,
{
    fn push(&mut self, probe: Probe<T::Code, By>) {
        if let Some(probe) = probe.guard(self.min, self.max) {
            self.bin.push(probe);
        }
    }
}

impl<'a, T, By, B> Iterator for Heap<'a, By, WaveletMatrix<T, B>>
where
    T: Word,
    B: Bits,
    Probe<T, By>: Ord,
{
    type Item = (usize, T);
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(Probe {
            index: (i, j),
            depth,
            mut value,
            _kind,
        }) = self.bin.pop()
        {
            if depth == T::DEPTH {
                return Some((j - i, value));
            }

            let fid = &self.seq.fids[depth];

            let rank0_beg = fid.rank0(..i);
            let rank0_end = fid.rank0(..j);
            let rank1_beg = i - rank0_beg;
            let rank1_end = j - rank0_end;

            if rank0_beg < rank0_end {
                let index = (rank0_beg, rank0_end);
                let depth = depth + 1;

                self.push(Probe {
                    index,
                    depth,
                    value,
                    _kind: PhantomData,
                });
            }

            if rank1_beg < rank1_end {
                value.put1(T::DEPTH - depth - 1);

                let index = {
                    let tip = self.seq.tips[depth];
                    (tip + rank1_beg, tip + rank1_end)
                };
                let depth = depth + 1;

                self.push(Probe {
                    index,
                    depth,
                    value,
                    _kind: PhantomData,
                });
            };
        }
        None
    }
}

mod cmpby {
    use super::{
        cmp::{Ordering, Reverse},
        Probe, Word,
    };

    use crate::ops::Code;

    impl<'a, By, T: Word> Probe<T, By> {
        pub(super) fn guard(self, min: Option<T>, max: Option<T>) -> Option<Probe<T, By>> {
            let prefix = |sym, d| sym >> (T::DEPTH - d);

            let val = prefix(self.value, self.depth);
            let min = min.map(|min| prefix(min, self.depth));
            let max = max.map(|max| prefix(max - T::_1, self.depth));

            match (min, max) {
                (Some(min), Some(max)) if min <= val && val <= max => Some(self),
                (Some(min), None) if min <= val => Some(self),
                (None, Some(max)) if val <= max => Some(self),
                (None, None) => Some(self),
                _ => None,
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Top;
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Min;
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Max;

    /// For Top
    impl<T: Word> PartialEq for Probe<T, Top> {
        fn eq(&self, that: &Probe<T, Top>) -> bool {
            let (i, j) = self.index;
            let (a, b) = that.index;
            j - i == b - a
        }
    }
    impl<T: Word> Eq for Probe<T, Top> {}

    impl<T: Word> PartialOrd for Probe<T, Top> {
        fn partial_cmp(&self, that: &Probe<T, Top>) -> Option<Ordering> {
            let len0 = self.index.1 - self.index.0;
            let len1 = that.index.1 - that.index.0;
            Some(if len0 != len1 {
                len0.cmp(&len1)
            } else if self.value != that.value {
                self.value.cmp(&that.value)
            } else {
                self.index.0.cmp(&that.index.0)
            })
        }
    }

    impl<T: Word> Ord for Probe<T, Top> {
        #[inline]
        fn cmp(&self, that: &Probe<T, Top>) -> Ordering {
            self.partial_cmp(that).unwrap()
        }
    }

    /// For Min
    impl<T: Word> PartialEq for Probe<T, Min> {
        fn eq(&self, that: &Probe<T, Min>) -> bool {
            self.value.eq(&that.value)
        }
    }
    impl<T: Word> Eq for Probe<T, Min> {}

    impl<T: Word> PartialOrd for Probe<T, Min> {
        fn partial_cmp(&self, that: &Probe<T, Min>) -> Option<Ordering> {
            Reverse(self.value).partial_cmp(&Reverse(that.value))
        }
    }
    impl<T: Word> Ord for Probe<T, Min> {
        fn cmp(&self, that: &Probe<T, Min>) -> Ordering {
            Reverse(self.value).cmp(&Reverse(that.value))
        }
    }

    /// For Max
    impl<T: Word> PartialEq for Probe<T, Max> {
        fn eq(&self, that: &Probe<T, Max>) -> bool {
            self.value.eq(&that.value)
        }
    }
    impl<T: Word> Eq for Probe<T, Max> {}

    impl<T: Word> PartialOrd for Probe<T, Max> {
        fn partial_cmp(&self, that: &Probe<T, Max>) -> Option<Ordering> {
            self.value.partial_cmp(&that.value)
        }
    }
    impl<T: Word> Ord for Probe<T, Max> {
        fn cmp(&self, that: &Probe<T, Max>) -> Ordering {
            self.value.cmp(&that.value)
        }
    }
}

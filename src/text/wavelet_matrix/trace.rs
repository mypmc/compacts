use std::iter::{Enumerate, FusedIterator};

use crate::{num::Word, ops::Bits, ops::Code};

use super::{Row, Rows, View, WaveletMatrix};

/// Traces nodes from top to bottom, invoking `F` for each depth to decide which route to trace.
#[derive(Debug, Clone)]
struct Trace<'a, B, F> {
    index: (usize, usize),
    rows: Enumerate<Rows<'a, B>>,
    router: F, // invoke for each depth to decide which route to trace
}

fn trace_by<B, F>(index: (usize, usize), rows: Rows<'_, B>, router: F) -> Trace<'_, B, F> {
    let rows = rows.enumerate();
    Trace {
        index,
        rows,
        router,
    }
}

fn by_value<B, T>(
    index: (usize, usize),
    rows: Rows<'_, B>,
    val: T,
) -> Trace<'_, B, impl FnMut(Data) -> Route>
where
    T: Code,
{
    trace_by(index, rows, move |Data { depth, .. }| {
        Route::from_bit(val.bit(T::DEPTH - depth - 1))
    })
}

impl<'a, T: Code, B> View<'a, WaveletMatrix<T, B>> {
    #[inline]
    fn trace(&self, val: T) -> Option<Trace<'a, B, impl FnMut(Data) -> Route>> {
        self.idx
            .as_ref()
            .map(|&idx| by_value(idx, self.seq.rows(), val))
    }

    #[inline]
    fn trace_by<F>(&self, router: F) -> Option<Trace<'a, B, F>> {
        self.idx
            .as_ref()
            .map(|&idx| trace_by(idx, self.seq.rows(), router))
    }
}

impl<'a, T: Code, B: Bits> View<'a, WaveletMatrix<T, B>> {
    #[inline]
    pub fn get(&self, i: usize) -> Option<T>
    where
        T: Word,
    {
        self.idx.as_ref().and_then(|(x, _y)| {
            let seq = self.seq;

            let mut i = i + x;
            if seq.size <= i {
                return None;
            }

            let mut sym = T::MIN;
            for (depth, fid) in seq.fids.iter().enumerate() {
                if fid.bit(i) {
                    sym.put1(T::DEPTH - depth - 1);
                    i = fid.rank1(..i) + seq.tips[depth]; // add count0 if b is 1
                } else {
                    i = fid.rank0(..i);
                }
            }
            Some(sym)
        })
    }

    /// ```
    /// use compacts::{BitArray, WaveletMatrix, ops::{Rank, Select}};
    /// let mut data = vec![5u8, 4, 5, 5, 2, 1, 5, 6, 1, 3, 5, 0]; // len=12
    /// let wm = WaveletMatrix::<u8, BitArray<u64>>::from(data.as_mut_slice());
    ///
    /// // min value in [2, 11)
    /// assert_eq!(wm.view(2..11).quantile(0), Some((0, 1)));
    /// // max value in [2, 11), 11 - 2 - 1
    /// assert_eq!(wm.view(2..11).quantile(8), Some((0, 6)));
    ///
    /// assert_eq!(wm.view(2..11).quantile(4), Some((0, 5))); // 1st 5
    /// assert_eq!(wm.select(&5, 0 + wm.rank(&5, ..2)), Some(2));
    ///
    /// assert_eq!(wm.view(2..11).quantile(5), Some((1, 5))); // 2nd 5
    /// assert_eq!(wm.select(&5, 1 + wm.rank(&5, ..2)), Some(3));
    ///
    /// assert_eq!(wm.view(2..11).quantile(6), Some((2, 5))); // 3rd 5
    /// assert_eq!(wm.select(&5, 2 + wm.rank(&5, ..2)), Some(6));
    ///
    /// assert_eq!(wm.view(2..11).quantile(7), Some((3, 5))); // 4th 5
    /// assert_eq!(wm.select(&5, 3 + wm.rank(&5, ..2)), Some(10));
    ///
    /// assert_eq!(wm.view(2..11).quantile(8), Some((0, 6))); // 1st 6
    /// assert_eq!(wm.view(2..11).quantile(9), None);
    ///
    /// ```
    #[inline]
    pub fn quantile(&self, mut k: usize) -> Option<(usize, T)>
    where
        T: Word,
    {
        self.idx.as_ref().and_then(|&idx| {
            let seq = self.seq;

            let mut sym = T::MIN;

            let rf = |data: Data| {
                if k < data.rank0 {
                    Route::Lhs
                } else {
                    sym.put1(T::DEPTH - data.depth - 1);
                    k -= data.rank0;
                    Route::Rhs
                }
            };

            let (i, j) = trace_by(idx, seq.rows(), rf).last().unwrap().index;
            if i + k < j {
                Some((k, sym))
            } else {
                None
            }
        })
    }

    /// Counts the occurences of `val` in this view.
    pub fn count(&self, val: &T) -> usize {
        self.trace(*val).map_or(0, |trace| {
            let (i, j) = trace.last().unwrap().index;
            j - i
        })
    }

    /// Counts the occurences of `val` in this view.
    ///
    /// ```
    /// use compacts::{BitArray, WaveletMatrix, ops::Select};
    /// let mut vec = vec![5u8, 4, 5, 5, 2, 1, 5, 6, 1, 3, 5, 0];
    /// let wm = WaveletMatrix::<u8, BitArray<u64>>::from(vec.as_mut_slice());
    ///
    /// // [ 5, 4, 5, 5, 2, 1, 5, 6, 1, 3, 5, 0 ]
    /// let view = wm.view(..);
    /// assert_eq!(view.counts(&5).lt, 6);
    /// assert_eq!(view.counts(&5).gt, 1);
    /// assert_eq!(view.counts(&5).eq, 5);
    ///
    /// // [ 5, 4, 5, [ 5, 2, 1, 5, 6, 1, 3, 5 ], 0 ]
    /// let view = wm.view(3..11);
    /// assert_eq!(view.counts(&5).lt, 4);
    /// assert_eq!(view.counts(&5).gt, 1);
    /// assert_eq!(view.counts(&5).eq, 3);
    /// ```
    pub fn counts(&self, val: &T) -> Counts {
        let mut lt = 0;
        let mut gt = 0;
        let eq = {
            let rf = |data: Data| {
                if val.bit(T::DEPTH - data.depth - 1) {
                    lt += data.rank0;
                    Route::Rhs
                } else {
                    gt += data.rank1;
                    Route::Lhs
                }
            };
            self.trace_by(rf).map_or(0, |trace| {
                let (i, j) = trace.last().unwrap().index;
                j - i
            })
        };

        Counts { eq, lt, gt }
    }

    /// ```
    /// use compacts::{BitArray, WaveletMatrix, ops::Select};
    /// let mut vec = vec![5u8, 4, 5, 5, 2, 1, 5, 6, 1, 3, 5, 0];
    /// let wm = WaveletMatrix::<u8, BitArray<u64>>::from(vec.as_mut_slice());
    ///
    /// {
    ///     // [ 5, 4, 5, 5, 2, 1, 5, 6, 1, 3, 5, 0 ]
    ///     let view = wm.view(..);
    ///     assert_eq!(view.select(&5, 0), Some(0));
    ///     assert_eq!(view.select(&5, 1), Some(2));
    ///     assert_eq!(view.select(&5, 2), Some(3));
    ///     assert_eq!(view.select(&5, 3), Some(6));
    ///     assert_eq!(view.select(&5, 4), Some(10));
    ///     assert_eq!(view.select(&5, 5), None);
    /// }
    ///
    /// {
    ///     // [ 5, 4, 5, [ 5, 2, 1, 5, 6, 1, 3, 5 ], 0 ]
    ///     let view = wm.view(3..11);
    ///     assert_eq!(view.select(&5, 0), Some(0));
    ///     assert_eq!(view.select(&5, 1), Some(3));
    ///     assert_eq!(view.select(&5, 2), Some(7));
    ///     assert_eq!(view.select(&4, 0), None);
    ///     assert_eq!(view.select(&4, 1), None);
    ///     assert_eq!(view.select(&2, 0), Some(1));
    ///     assert_eq!(view.select(&1, 0), Some(2));
    ///     assert_eq!(view.select(&1, 1), Some(5));
    ///     assert_eq!(view.select(&1, 2), None);
    /// }
    /// ```
    #[inline]
    pub fn select(&self, val: &T, n: usize) -> Option<usize> {
        self.idx.as_ref().and_then(|&idx| {
            let seq = self.seq;

            let (i, j) = by_value(idx, seq.rows(), *val).last().unwrap().index;
            if j - i <= n {
                None
            } else {
                let mut pos = i + n;
                for depth in (0..T::DEPTH).rev() {
                    let fid = &seq.fids[depth];
                    let bit = val.bit(T::DEPTH - depth - 1);
                    let tip = seq.tips[depth] * (bit as usize);
                    pos = fid.select(bit, pos - tip).unwrap();
                }
                Some(pos - idx.0)
            }
        })
    }
}

// impl<'a, T: Code, B> View<'a, WaveletMatrix<T, B>> {
//     #[inline]
//     pub fn select(&self, val: &T, nth: usize) -> Option<usize>
//     where
//         B: Select1 + Select0,
//     {
//         self.as_ref().and_then(|seq| {
//             let fids = &seq.inner.fids;
//             let tips = &seq.inner.tips;
//             let rec = SelectHelper { nth, fids, tips };
//             let mut trace = seq.trace(*val);
//             rec.select(&mut trace).map(|pos| pos - seq.index.0)
//         })
//     }
// }
//
// struct SelectHelper<'a, B> {
//     nth: usize,
//     fids: &'a [B],
//     tips: &'a [usize],
// }
// impl<'a, B> SelectHelper<'a, B>
// where
//     B: Select1 + Select0,
// {
//     fn select<S, F>(&self, tr: &mut TraceBy<'a, B, F>) -> Option<usize>
//     where
//         S: Code,
//         F: FnMut(Data) -> Route,
//     {
//         let Node {
//             depth,
//             route,
//             index: (i, j),
//         } = tr.next().expect("unreachable");

//         let bit = route.to_bit();
//         let fid = &self.fids[depth - 1];
//         let tip = (bit as usize) * self.tips[depth - 1];

//         if depth == S::DEPTH {
//             if j - i <= self.nth {
//                 None
//             } else {
//                 Some(fid.select(&bit, i + self.nth - tip).unwrap())
//             }
//         } else {
//             self.select(tr).and_then(|pos| fid.select(&bit, pos - tip))
//         }
//     }
// }

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct Node {
    depth: usize,
    index: (usize, usize),
    route: Route, // node direction from depth-1 to depth
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct Data {
    depth: usize,
    rank0: usize,
    rank1: usize,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Route {
    Lhs,
    Rhs,
}
impl Route {
    #[inline]
    fn from_bit(bit: bool) -> Route {
        if bit {
            Route::Rhs
        } else {
            Route::Lhs
        }
    }

    // #[inline]
    // fn to_bit(&self) -> bool {
    //     match self {
    //         Route::Lhs => false,
    //         Route::Rhs => true,
    //     }
    // }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct Counts {
    /// Count of the occurences of value less than `value`.
    pub lt: usize,

    /// Count of the occurences of `value`.
    pub eq: usize,

    /// Count of the occurences of value greater than `value`
    pub gt: usize,
}

impl<'a, B, F> Iterator for Trace<'a, B, F>
where
    B: Bits,
    F: FnMut(Data) -> Route,
{
    type Item = Node;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.rows.next().map(|(depth, Row { fid, tip })| {
            let (i, j) = self.index;

            let rank0_bpos = fid.rank0(..i);
            let rank0_epos = fid.rank0(..j);
            let rank1_bpos = i - rank0_bpos;
            let rank1_epos = j - rank0_epos;

            let rank0 = rank0_epos - rank0_bpos;
            let rank1 = rank1_epos - rank1_bpos;

            match (self.router)(Data {
                depth,
                rank0,
                rank1,
            }) {
                Route::Lhs => {
                    self.index = (rank0_bpos, rank0_epos);
                    Node {
                        route: Route::Lhs,
                        depth: depth + 1,
                        index: self.index,
                    }
                }
                Route::Rhs => {
                    self.index = (rank1_bpos + tip, rank1_epos + tip);
                    Node {
                        route: Route::Rhs,
                        depth: depth + 1,
                        index: self.index,
                    }
                }
            }
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.rows.size_hint()
    }

    // #[inline]
    // fn nth(&mut self, i: usize) -> Option<Self::Code> {
    //     self.matrix.nth(i).map(|t| {
    //         let node = self.next_node(t);
    //         self.index = node.index;
    //         node
    //     })
    // }
}

impl<'a, B: Bits, F> ExactSizeIterator for Trace<'a, B, F>
where
    F: FnMut(Data) -> Route,
{
    #[inline]
    fn len(&self) -> usize {
        self.rows.len()
    }
}

impl<'a, B: Bits, F> FusedIterator for Trace<'a, B, F> where F: FnMut(Data) -> Route {}

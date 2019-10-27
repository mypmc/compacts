#![allow(missing_docs)]
#![allow(unused_imports)]

pub mod wavelet_matrix;
pub mod wavelet_tree;

#[doc(inline)]
pub use wavelet_matrix::WaveletMatrix;
#[doc(inline)]
pub use wavelet_tree::WaveletTree;

/// View represents an immutable view `[i, j)` of `T`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct View<'a, T: ?Sized> {
    pub(crate) idx: Option<(usize, usize)>, // None if i >= j
    pub(crate) seq: &'a T,
}

impl<T: crate::num::Word> crate::ops::Code for T {
    const DEPTH: usize = T::BITS;
    const MIN: Self = T::NONE;
    const MAX: Self = T::FULL;
}

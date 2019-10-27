//! `compacts` implements succinct data structures

// [Broadword](https://www.semanticscholar.org/paper/Broadword-Implementation-of-Rank%2FSelect-Queries-Vigna/2c530ff32d6177e0e945f121f992431cf035112b)
// [PTSelect](https://pdfs.semanticscholar.org/7140/dfc69ed2ca65dd8bbdcf5d5b3742f2d839c2.pdf)
//
// [Poppy](https://www.cs.cmu.edu/~dga/papers/zhou-sea2013.pdf)
// [Optimized Succinct Data Structures](https://people.eng.unimelb.edu.au/sgog/optimized.pdf)

// #![deny(warnings)]
// #![deny(missing_docs)]

macro_rules! BOUNDS_CHECK {
    ($cond:expr) => {
        assert!($cond, "out of bounds");
    };
}

pub mod bits;
pub mod text;

pub mod ops;

pub mod num;

mod fenwick;
mod union_find;

pub use bits::map::BitMap;
pub use bits::pop_vec::Pop;

#[doc(inline)]
pub use bits::{bit_array::BitArray, bit_vec::BitVec};

#[doc(inline)]
pub use text::wavelet_matrix::WaveletMatrix;

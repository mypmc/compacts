use std::marker::PhantomData;

use crate::{
    bits::{blocks_by, Words},
    ops::*,
    // BitMap,
};

/// `WaveletTree`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaveletTree<T, B> {
    code: PhantomData<T>,
    tree: Vec<Node<B>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Node<B> {
    size: usize,
    dict: B,
}

// /// `Buf<T>`
// #[derive(Debug, Clone, PartialEq, Eq)]
// struct Buf<T> {
//     size: usize,  // actual size of bits; not `bits.len() * T::SIZE`
//     data: Vec<T>, // bit blocks
// }

// impl<B: FixedBits> Node<Vec<B>> {
//     fn with_capacity(min_cap: usize) -> Self {
//         Node {
//             size: 0,
//             dict: vec![B::none(); blocks(min_cap, B::SIZE)],
//         }
//     }

//     fn push(&mut self, b: bool) {
//         let i = self.size;
//         if self.dict.size() <= i {
//             self.dict.resize_with(blocks(i, B::SIZE), B::none);
//         }
//         if b {
//             self.dict.put1(i);
//         }
//         // else {
//         //     self.data.put0(i);
//         // }
//         self.size += 1;
//     }

//     // pub fn pop(&mut self) -> Option<bool> {
//     //     if self.size == 0 {
//     //         None
//     //     } else {
//     //         let pos = self.size - 1;
//     //         if self.dict.bit(pos) {
//     //             self.dict.put0(pos);
//     //             self.size -= 1;
//     //             Some(true)
//     //         } else {
//     //             self.size -= 1;
//     //             Some(false)
//     //         }
//     //     }
//     // }
// }

// impl<T: FixedBits> Buf<T> {
//     fn with_capacity(min_cap: usize) -> Self {
//         let data = vec![T::none(); blocks(min_cap, T::SIZE)];
//         Buf { size: 0, data }
//     }

//     fn push(&mut self, b: bool) {
//         let i = self.size;
//         if self.data.size() <= i {
//             self.data.resize_with(blocks(i, T::SIZE), T::none);
//         }
//         if b {
//             self.data.put1(i);
//         }
//         // else {
//         //     self.data.put0(i);
//         // }
//         self.size += 1;
//     }
// }

// impl<T: Words> Buf<Option<Box<T>>> {
//     fn into_node(mut self) -> Node<BitMap<T>> {
//         let size = self.size;
//         self.data.truncate(blocks(size, T::SIZE));
//         let dict = BitMap::from_vec(self.data);
//         Node { size, dict }
//     }
// }

// impl<T, B> From<&[T]> for WaveletTree<T, BitMap<B>>
// where
//     T: Code,
//     B: Words,
// {
//     fn from(slice: &[T]) -> Self {
//         let mut tree = vec![Buf::with_capacity(0); T::DEPTH * 2 + 1];

//         for code in slice {
//             let mut pos = 0;
//             let mut bit = 0;

//             while pos < tree.len() {
//                 let b = code.bit(bit);
//                 tree[pos].push(b);

//                 pos = pos * 2 + 1 + (b as usize);
//                 bit = bit + 1;
//             }
//         }

//         WaveletTree {
//             code: PhantomData,
//             tree: tree.into_iter().map(Buf::into_node).collect(),
//         }
//     }
// }

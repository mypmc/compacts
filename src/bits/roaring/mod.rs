#![allow(missing_docs, unused_imports)]
#![allow(warnings)]

// mod map;
// mod posn;
// mod repr;

mod locs;
mod repr;
mod runs;

// mod size_of {
//     use super::Run;
//     pub static U16: usize = std::mem::size_of::<u16>();
//     pub static U32: usize = std::mem::size_of::<u32>();
//     pub static U64: usize = std::mem::size_of::<u64>();
//     pub static RUN: usize = std::mem::size_of::<Run>();
// }

// #[cfg(test)]
// mod tests;

use std::cmp::Ordering::{self, Equal as EQ, Greater as GT, Less as LT};

use crate::{num::try_cast, ops::*};

// #[cfg(test)]
// pub(crate) use {posn::Pos1, runs::Runs};

// #[derive(Debug, Clone, PartialEq, Eq)]
// pub struct BitMap<K> {
//     size: usize,
//     keys: Vec<K>,
//     data: Vec<Block>,
// }

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Block(Repr);

impl Block {
    const BITS: usize = 65536;
}

#[derive(Clone)]
enum Repr {
    Page(Page), // dense
    Loc1(Loc1), // default, sparse
    Runs(Runs), // need explicit conversion
}

type Page = Box<[u64; 1024]>;

/// 1-based sorted bit sequence.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Loc1 {
    data: Vec<u16>,
}

// // /// 0-based sorted bit sequence.
// // #[derive(Debug, Clone, Default, PartialEq, Eq)]
// // pub(crate) struct Loc0 {
// //     locs: Vec<u16>,
// // }

/// A run length encoded bits.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Runs {
    data: Vec<Run>,
}

/// `Run` is an inclusive range between `[i, j]`.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct Run(u16, u16);

impl From<&'_ Loc1> for Page {
    fn from(pos1: &Loc1) -> Self {
        let mut bits = Page::none();
        for &i in &pos1.data {
            bits.put1(try_cast(i));
        }
        bits
    }
}

// impl From<&'_ Runs> for BitsRepr {
//     fn from(runs: &Runs) -> Self {
//         let mut bits = BitsRepr::empty();
//         for &Bounds(i, j) in runs {
//             for b in i..=j {
//                 bits.put1(try_cast::<u16, usize>(b));
//             }
//         }
//         bits
//     }
// }

// impl From<&'_ BitsRepr> for Loc1 {
//     fn from(bits: &BitsRepr) -> Self {
//         let mut data = Vec::with_capacity(1 << 10);
//         for (i, &w) in bits.iter().enumerate().filter(|&(_, &v)| v != 0) {
//             for p in 0..u64::SIZE {
//                 if w.get(p) {
//                     data.push(try_cast::<usize, u16>(i * u64::BITS + p));
//                 }
//             }
//         }
//         data.shrink_to_fit();
//         Loc1 { data }
//     }
// }

// impl From<&'_ Runs> for Loc1 {
//     fn from(runs: &Runs) -> Self {
//         let data = runs
//             .data
//             .iter()
//             .flat_map(|Bounds(s, e)| *s..=*e)
//             .collect::<Vec<_>>();
//         Loc1 { data }
//     }
// }

// impl From<&'_ Loc1> for Runs {
//     #[inline]
//     fn from(loc1: &'_ Loc1) -> Self {
//         Runs {
//             data: loc1.runs().collect(),
//         }
//     }
// }

// // impl From<&'_ BitsRepr> for Runs {
// //     fn from(bits: &'_ BitsRepr) -> Self {
// //         let mut runs = Runs::new();
// //         for (i, &w) in bits.iter().enumerate().filter(|&(_, &v)| v != 0) {
// //             for p in 0..u64::SIZE {
// //                 if w.bit(p) {
// //                     runs.put1(try_cast::<usize, u64>(i) * u64::SIZE + p);
// //                 }
// //             }
// //         }
// //         runs
// //     }
// // }

// // impl mask::Union<Loc1> for BitsRepr {
// //     #[inline]
// //     fn union(&mut self, pos1: &Loc1) {
// //         for &b in pos1 {
// //             self.put1(try_cast(b));
// //         }
// //     }
// // }

// // impl mask::Union<Runs> for BitsRepr {
// //     #[inline]
// //     fn union(&mut self, pos1: &Runs) {
// //         for &Bounds(i, j) in pos1 {
// //             self.set1(try_cast::<u16, u64>(i)..=try_cast(j));
// //         }
// //     }
// // }

// // impl mask::Difference<Loc1> for BitsRepr {
// //     #[inline]
// //     fn difference(&mut self, pos1: &Loc1) {
// //         for &b in pos1 {
// //             self.put0(u64::from(b));
// //         }
// //     }
// // }

// // impl mask::Difference<Runs> for BitsRepr {
// //     #[inline]
// //     fn difference(&mut self, runs: &Runs) {
// //         for Bounds(n, m) in runs {
// //             self.set0(try_cast::<u16, u64>(*n)..=try_cast(*m));
// //         }
// //     }
// // }

// // impl mask::SymmetricDifference<Loc1> for BitsRepr {
// //     #[inline]
// //     fn symmetric_difference(&mut self, pos1: &Loc1) {
// //         for &b in pos1 {
// //             let b = try_cast(b);
// //             if self.put1(b) {
// //                 self.put0(b);
// //             }
// //         }
// //     }
// // }

// // impl mask::SymmetricDifference<Runs> for BitsRepr {
// //     #[inline]
// //     fn symmetric_difference(&mut self, runs: &Runs) {
// //         for &Bounds(i, j) in runs {
// //             self.flip(try_cast::<u16, u64>(i)..=try_cast(j));
// //         }
// //     }
// // }

// // impl mask::Intersection<BitsRepr> for Loc1 {
// //     #[inline]
// //     fn intersection(&mut self, bits: &BitsRepr) {
// //         self.0.retain(|&x| bits.get(u64::from(x)));
// //     }
// // }
// // impl mask::Intersection<Runs> for Loc1 {
// //     #[inline]
// //     fn intersection(&mut self, runs: &Runs) {
// //         self.0.retain(|&x| runs.get(u64::from(x)));
// //     }
// // }

// // impl mask::Difference<BitsRepr> for Loc1 {
// //     #[inline]
// //     fn difference(&mut self, bits: &BitsRepr) {
// //         self.0.retain(|&x| bits.get(u64::from(x)));
// //     }
// // }
// // impl mask::Difference<Runs> for Loc1 {
// //     #[inline]
// //     fn difference(&mut self, runs: &Runs) {
// //         self.0.retain(|&x| runs.get(u64::from(x)));
// //     }
// // }

// // pub struct Steps<'a, T> {
// //     iter: std::iter::Enumerate<std::slice::Iter<'a, T>>,
// // }

// // impl<'a> mask::BitMask for &'a crate::BitsRepr<Loc1> {
// //     type Index = usize;
// //     type Value = Cow<'a, Loc1>;
// //     type Steps = Steps<'a, Loc1>;
// //     fn into_steps(self) -> Self::Steps {
// //         Steps {
// //             iter: self.iter().enumerate(),
// //         }
// //     }
// // }

// // impl<'a> mask::BitMask for &'a crate::BitsRepr<Runs> {
// //     type Index = usize;
// //     type Value = Cow<'a, Runs>;
// //     type Steps = Steps<'a, Runs>;
// //     fn into_steps(self) -> Self::Steps {
// //         Steps {
// //             iter: self.iter().enumerate(),
// //         }
// //     }
// // }

// // impl<'a> mask::BitMask for &'a crate::BitsRepr<Block> {
// //     type Index = usize;
// //     type Value = Cow<'a, Block>;
// //     type Steps = Steps<'a, Block>;
// //     fn into_steps(self) -> Self::Steps {
// //         Steps {
// //             iter: self.iter().enumerate(),
// //         }
// //     }
// // }

// // impl<'a> Iterator for Steps<'a, Loc1> {
// //     type Item = (usize, Cow<'a, Loc1>);
// //     fn next(&mut self) -> Option<Self::Item> {
// //         self.iter.find_map(|(index, pos1)| {
// //             if !pos1.is_empty() {
// //                 let value = Cow::Borrowed(pos1);
// //                 Some((index, value))
// //             } else {
// //                 None
// //             }
// //         })
// //     }
// // }

// // impl<'a> Iterator for Steps<'a, Runs> {
// //     type Item = (usize, Cow<'a, Runs>);
// //     fn next(&mut self) -> Option<Self::Item> {
// //         self.iter.find_map(|(index, runs)| {
// //             if !runs.is_empty() {
// //                 let value = Cow::Borrowed(runs);
// //                 Some((index, value))
// //             } else {
// //                 None
// //             }
// //         })
// //     }
// // }

// // impl<'a> Iterator for Steps<'a, Block> {
// //     type Item = (usize, Cow<'a, Block>);
// //     fn next(&mut self) -> Option<Self::Item> {
// //         self.iter.find_map(|(index, b)| {
// //             if b.any() {
// //                 Some((index, Cow::Borrowed(b)))
// //             } else {
// //                 None
// //             }
// //         })
// //     }
// // }

// // #[derive(Debug, Clone, PartialEq, Eq)]
// // pub struct Bytes<T> {
// //     header: Header,
// //     bytes: T,
// // }

// // #[derive(Debug, Clone, PartialEq, Eq)]
// // enum Header {
// //     // SERIAL_COOKIE && blocks < NO_OFFSET_THRESHOLD
// //     Inline(BitMap),
// //     // SERIAL_COOKIE && blocks >= NO_OFFSET_THRESHOLD
// //     Serial {
// //         runs: Vec<u8>,
// //         keys: Vec<u16>,
// //         pops: Vec<u32>,
// //         locs: Vec<u32>,
// //     },
// //     // SERIAL_NO_RUN
// //     NoRuns {
// //         keys: Vec<u16>,
// //         pops: Vec<u32>,
// //         locs: Vec<u32>,
// //     },
// // }

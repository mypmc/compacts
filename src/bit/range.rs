// use std::{borrow::Cow, marker::PhantomData};

// use crate::{bits, bits::*};

// pub trait BitsIndex<T>: crate::private::Sealed {
//     type Output;
//     fn get(&self, target: &T) -> Self::Output;
// }

// /// All types that implements `Access` can be indexed by `u64`.
// impl<T: Access> BitsIndex<T> for u64 {
//     type Output = bool;
//     #[inline]
//     fn get(&self, bv: &T) -> bool {
//         bv.access(*self)
//     }
// }

// #[derive(Clone, Debug, PartialEq, Eq)]
// pub struct Range<'a, T> {
//     // Range represents `bits & mask`.
//     // If mask is empty, there are no bits in this range.
//     bits: &'a T,
//     mask: Option<Mask>,
// }

// #[derive(Copy, Clone, Debug, PartialEq, Eq)]
// pub(crate) struct Mask {
//     index: u64,
//     len: u64,
// }

// impl Mask {
//     fn new(index: u64, len: u64) -> Self {
//         // assert!(index + len <= bits::MAX_BITS);
//         Mask { index, len }
//     }

//     fn contains(&self, i: u64) -> bool {
//         self.index <= i && i < self.index + self.len
//     }

//     fn overlap(&self, that: &Mask) -> bool {
//         self.index < that.index + that.len && that.index < self.index + self.len
//     }
// }

// // impl<'a, T> Range<'a, T> {
// //     fn mask<U: Into<Option<Mask>>>(bits: &'a T, data: U) -> Self {
// //         let mask = data.into();
// //         Range { bits, mask }
// //     }

// //     pub fn get<'r, Ix: BitsIndex<&'r Self>>(&'r self, index: Ix) -> Ix::Output {
// //         index.get(self)
// //     }
// // }

// // fn intersect<'a, 'b, A, B>(this: A, that: B) -> Option<Mask>
// // where
// //     A: Into<Option<&'a Mask>>,
// //     B: Into<Option<&'b Mask>>,
// // {
// //     match (this.into(), that.into()) {
// //         (Some(this), Some(that)) => {
// //             if this.overlap(that) {
// //                 let idx = std::cmp::max(this.index, that.index);
// //                 let len = std::cmp::min(this.index + this.len, that.index + that.len) - idx;
// //                 Some(Mask::new(idx, len))
// //             } else {
// //                 None
// //             }
// //         }
// //         _ => None,
// //     }
// // }

// // impl<'a, T> BitsIndex<Range<'a, T>> for ops::RangeFull {
// //     type Output = Range<'a, T>;
// //     fn get(&self, range: &Range<'a, T>) -> Self::Output {
// //         let bits = range.bits;
// //         let mask = range.mask;
// //         Range::mask(bits, mask)
// //     }
// // }

// // impl<'a, T: Count> BitsIndex<Range<'a, T>> for ops::Range<u64> {
// //     type Output = Range<'a, T>;
// //     fn get(&self, range: &Range<'a, T>) -> Self::Output {
// //         let bits = range.bits.bits();
// //         let start = self.start;
// //         let end = self.end;

// //         if start <= end && start < bits && end <= bits {
// //             let this = range.mask;
// //             let that = Mask::new(start, end - start);
// //             Range::mask(range.bits, intersect(&this, &that))
// //         } else {
// //             Range::mask(range.bits, None)
// //         }
// //     }
// // }

// // impl<'a, T: Count> BitsIndex<Range<'a, T>> for ops::RangeFrom<u64> {
// //     type Output = Range<'a, T>;
// //     fn get(&self, range: &Range<'a, T>) -> Self::Output {
// //         let bits = range.bits.bits();
// //         let start = self.start;
// //         let end = bits;

// //         if start <= end && start < bits {
// //             let this = range.mask;
// //             let that = Mask::new(start, end - start);
// //             Range::mask(range.bits, intersect(&this, &that))
// //         } else {
// //             Range::mask(range.bits, None)
// //         }
// //     }
// // }

// // impl<'a, T: Count> BitsIndex<Range<'a, T>> for ops::RangeTo<u64> {
// //     type Output = Range<'a, T>;
// //     fn get(&self, range: &Range<'a, T>) -> Self::Output {
// //         let bits = range.bits.bits();
// //         let start = 0;
// //         let end = self.end;

// //         if start <= end && start < bits && end <= bits {
// //             let this = range.mask;
// //             let that = Mask::new(start, end - start);
// //             Range::mask(range.bits, intersect(&this, &that))
// //         } else {
// //             Range::mask(range.bits, None)
// //         }
// //     }
// // }

// // // This can be implemented using generics if rust has specialization.
// // macro_rules! implBitsIndex {
// //     ($( [ $($tts:tt)* ] for $Type:ty ; )+) => {
// //         $(
// //             impl<'a, $($tts)*> BitsIndex<$Type> for ops::RangeFull {
// //                 type Output = Range<'a, $Type>;
// //                 fn get(&self, bits: &$Type) -> Self::Output {
// //                     Range::mask(bits, Mask::new(0, bits.bits()))
// //                 }
// //             }

// //             impl<'a, $($tts)*> BitsIndex<&'a $Type> for ops::Range<u64> {
// //                 type Output = Range<'a, $Type>;
// //                 fn get(&self, bits: &'a $Type) -> Self::Output {
// //                     let size = bits.bits();
// //                     let start = self.start;
// //                     let end = self.end;

// //                     if start <= end && start < size && end <= size {
// //                         Range::mask(bits, Mask::new(start, end - start))
// //                     } else {
// //                         Range::mask(bits, None)
// //                     }
// //                 }
// //             }

// //             impl<'a, $($tts)*> BitsIndex<&'a $Type> for ops::RangeFrom<u64> {
// //                 type Output = Range<'a, $Type>;
// //                 fn get(&self, bits: &'a $Type) -> Self::Output {
// //                     let size = bits.bits();
// //                     let start = self.start;
// //                     let end = size;

// //                     if start <= end && start < size {
// //                         Range::mask(bits, Mask::new(start, end - start))
// //                     } else {
// //                         Range::mask(bits, None)
// //                     }
// //                 }
// //             }

// //             impl<'a, $($tts)*> BitsIndex<&'a $Type> for ops::RangeTo<u64> {
// //                 type Output = Range<'a, $Type>;
// //                 fn get(&self, bits: &'a $Type) -> Self::Output {
// //                     let size = bits.bits();
// //                     let start = 0;
// //                     let end = self.end;

// //                     if start <= end && start < size && end <= size {
// //                         Range::mask(bits, Mask::new(start, end - start))
// //                     } else {
// //                         Range::mask(bits, None)
// //                     }
// //                 }
// //             }
// //         )+
// //     }
// // }
// // implBitsIndex!(
// //     [K: UnsignedInt, V: FiniteBits] for bits::EntryMap<K, V>;
// // );

// impl<'a, T: Access> Access for Range<'a, T> {
//     fn access(&self, i: u64) -> bool {
//         self.mask
//             .as_ref()
//             .map_or(false, |mask| mask.contains(i) && self.bits.access(i))
//     }
// }

// impl<'a, T: Rank> Count for Range<'a, T> {
//     fn bits(&self) -> u64 {
//         self.bits.bits()
//     }

//     fn count1(&self) -> u64 {
//         self.mask.as_ref().map_or(0, |mask| {
//             if mask.len == 0 {
//                 0
//             } else {
//                 let i = mask.index;
//                 let j = mask.index + mask.len;
//                 self.bits.rank1(j) - self.bits.rank1(i)
//             }
//         })
//     }
// }

// impl<'a, T: Rank> Rank for Range<'a, T> {
//     fn rank1(&self, i: u64) -> u64 {
//         self.mask.as_ref().map_or(0, |mask| {
//             if mask.len == 1 || i <= mask.index {
//                 0
//             } else if mask.index < i && i <= mask.index + mask.len {
//                 self.bits.rank1(i) - self.bits.rank1(mask.index)
//             } else {
//                 let i = mask.index;
//                 let j = mask.index + mask.len;
//                 self.bits.rank1(j) - self.bits.rank1(i)
//             }
//         })
//     }
// }

// pub struct RangeIntoIter<I: Iterator, A> {
//     bits: std::iter::Peekable<I>,
//     mask: std::iter::Peekable<Entries<usize, u64>>,
//     _ty: PhantomData<A>,
// }

// impl<'a, T, A: FiniteBits> IntoIterator for Range<'a, T>
// where
//     &'a T: IntoIterator<Item = A>,
//     RangeIntoIter<<&'a T as IntoIterator>::IntoIter, A>: Iterator<Item = A>,
// {
//     type Item = A;
//     type IntoIter = RangeIntoIter<<&'a T as IntoIterator>::IntoIter, A>;
//     fn into_iter(self) -> Self::IntoIter {
//         let bits = self.bits.into_iter().peekable();
//         let mask = if let Some(mask) = self.mask {
//             mask.entries(cast(A::BITS)).peekable()
//         } else {
//             Entries(None).peekable()
//         };
//         RangeIntoIter {
//             bits,
//             mask,
//             _ty: PhantomData,
//         }
//     }
// }

// macro_rules! implIterator {
//     ($Type:ty, $ctor:path) => {
//         impl<'a, I, K: UnsignedInt> Iterator for RangeIntoIter<I, Entry<K, Cow<'a, $Type>>>
//         where
//             I: Iterator<Item = Entry<K, Cow<'a, $Type>>>,
//         {
//             type Item = Entry<K, Cow<'a, $Type>>;
//             fn next(&mut self) -> Option<Self::Item> {
//                 use std::{cmp::Ordering, ops::BitAndAssign};

//                 let lhs = &mut self.bits;
//                 let rhs = &mut self.mask;

//                 loop {
//                     let cmp = lhs
//                         .peek()
//                         .and_then(|x| rhs.peek().map(|y| cast::<K, usize>(x.index).cmp(&y.index)));
//                     match cmp {
//                         Some(Ordering::Equal) => {
//                             let mut lhs = lhs.next().unwrap();
//                             let rhs = rhs.next().unwrap();
//                             lhs.value.to_mut().0.bitand_assign(&$ctor(rhs.value));
//                             break Some(lhs);
//                         }
//                         Some(Ordering::Less) => {
//                             lhs.next();
//                         }
//                         Some(Ordering::Greater) => {
//                             rhs.next();
//                         }
//                         None => break None,
//                     }
//                 }
//             }
//         }
//     };
// }

// fn to_array(vec: Vec<u64>) -> bits::encode::Map<u64> {
//     bits::encode::Map::from(vec)
// }
// fn to_block(vec: Vec<u64>) -> bits::encode::Encode {
//     bits::encode::Encode::Map(bits::encode::Map::from(vec))
// }

// implIterator!(Array, to_array);
// implIterator!(RoaringBlock, to_block);

// pub(crate) struct Entries<K: UnsignedInt, T>(
//     Option<Box<dyn Iterator<Item = Entry<K, Vec<T>>> + 'static>>,
// );

// impl<K: UnsignedInt, T: UnsignedInt> Iterator for Entries<K, T> {
//     type Item = Entry<K, Vec<T>>;
//     fn next(&mut self) -> Option<Self::Item> {
//         self.0.as_mut().and_then(|i| i.next())
//     }
// }

// impl Mask {
//     pub(crate) fn entries<K, T>(&self, chunk: usize) -> Entries<K, T>
//     where
//         K: UnsignedInt,
//         T: UnsignedInt,
//         [T]: Assign<std::ops::Range<u64>>,
//     {
//         assert!(chunk >= T::BITS as usize && chunk % T::BITS as usize == 0);

//         if self.len == 0 {
//             return Entries(None);
//         }

//         let i = self.index;
//         let j = self.index + self.len;

//         let (head_index, head_offset) = divmod::<K>(i, chunk as u64);
//         let (last_index, last_offset) = divmod::<K>(j, chunk as u64);
//         debug_assert!(head_index <= last_index);

//         let vec_len = chunk / T::BITS as usize;

//         Entries(Some(if head_index == last_index {
//             // one entry only
//             let mut vec = vec![!T::ZERO; vec_len];
//             vec.set0(0..head_offset);
//             vec.set0(last_offset..chunk as u64);
//             Box::new(std::iter::once(Entry::new(head_index, vec)))
//         } else {
//             let mut head = vec![!T::ZERO; vec_len];
//             head.set0(0..head_offset);
//             Box::new(
//                 std::iter::once(Entry::new(cast(head_index), head))
//                     .chain(
//                         (cast::<K, usize>(head_index) + 1..cast(last_index))
//                             .map(move |i| Entry::new(cast::<usize, K>(i), vec![!T::ZERO; vec_len])),
//                     )
//                     .chain({
//                         // let mut last = vec![!T::ZERO; vec_len];
//                         // last.remove_range(last_offset, chunk as u64 - last_offset);
//                         let mut last = vec![T::ZERO; vec_len];
//                         last.set1(0..last_offset);
//                         std::iter::once(Entry::new(cast(last_index), last))
//                     }),
//             )
//         }))
//     }
// }

// // impl<'a, T, U: UnsignedInt> IntoIterator for Range<'a, T>
// // where
// //     &'a T: IntoIterator<Item = Cow<'a, Entry<U, RoaringBlock>>>,
// // {
// //     type Item = <&'a T as IntoIterator>::Item;
// //     type IntoIter = AndIntoIter<
// //         <&'a T as IntoIterator>::IntoIter,
// //         Entries<'a, U, u64>,
// //         Cow<'a, Entry<U, RoaringBlock>>,
// //     >;

// //     fn into_iter(self) -> Self::IntoIter {
// //         AndIntoIter {
// //             lhs: self.bits.into_iter().peekable(),
// //             rhs: self.mask.entries::<U, u64>(RoaringBlock::SIZE).peekable(),
// //             _ty: PhantomData,
// //         }
// //     }
// // }

// #[cfg(test)]
// #[test]
// fn entries() {
//     macro_rules! check {
//         ($size:expr, $mask:expr) => {
//             let count = $mask.len;
//             let mut accum = 0;
//             for entry in $mask.entries::<usize, u64>($size) {
//                 accum += entry.value.count1();
//                 for w in &entry.value {
//                     println!("({:2}, {:064b})", entry.index, w);
//                 }
//             }
//             assert_eq!(count, accum);
//             println!("");
//         };
//     }

//     check!(64, Mask::new(64_u64, 64 * 1));
//     check!(64, Mask::new(64_u64, 64 * 2));
//     check!(64, Mask::new(64_u64, 64 * 3));
//     check!(64, Mask::new(64_u64, 64 * 4));
//     check!(64, Mask::new(64_u64, 64 * 5));

//     check!(128, Mask::new(60_u64, 72));
//     check!(128, Mask::new(61_u64, 132));
//     check!(128, Mask::new(62_u64, 1105));

//     check!(256, Mask::new(64_u64, 64 * 1));
//     check!(256, Mask::new(64_u64, 64 * 2));
//     check!(256, Mask::new(64_u64, 64 * 3));
//     check!(256, Mask::new(64_u64, 64 * 4));
//     check!(256, Mask::new(64_u64, 64 * 5));

//     check!(1024, Mask::new(65536_u64 * 3, 0));
//     check!(1024, Mask::new(65536_u64 * 3, 65536));
//     check!(1024, Mask::new(12384_u64 * 3, 65536));
//     check!(1024, Mask::new(16384_u64 * 3, 1_000_000));
//     check!(1024, Mask::new(65536_u64 * 3, 10000));
//     check!(1024, Mask::new(65536_u64 * 3, 1_000_000));
//     check!(2048, Mask::new(65536_u64, 65536));
//     check!(2048, Mask::new(12384_u64, 65536));
// }

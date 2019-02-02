// use std::{borrow::Cow, iter::Peekable, marker::PhantomData, ops};

// use crate::bits::*;

// /// Pad `I` with default value.
// pub struct PadUsingDefault<I: Iterator, T> {
//     value: Peekable<I>,
//     range: ops::Range<u64>, // dummy iterator
//     _item: PhantomData<T>,
// }

// impl<I: Iterator, T> PadUsingDefault<I, T> {
//     pub fn pad_using_default(range: ops::Range<u64>, iter: I) -> Self {
//         let value = iter.peekable();
//         let _item = PhantomData;
//         PadUsingDefault {
//             value,
//             range,
//             _item,
//         }
//     }
// }

// enum Item<K> {
//     Dummy(K),
//     Found,
//     Empty,
// }

// impl<'a, I, K, V> Iterator for PadUsingDefault<I, Entry<K, Cow<'a, V>>>
// where
//     K: Uint,
//     V: FiniteBits + Clone,
//     I: Iterator<Item = Entry<K, Cow<'a, V>>>,
// {
//     type Item = Entry<K, Cow<'a, V>>;

//     fn next(&mut self) -> Option<Self::Item> {
//         let item = match (self.range.next(), self.value.peek()) {
//             (Some(k), Some(p)) => {
//                 let k = cast::<u64, K>(k);
//                 if k < p.index {
//                     Item::Dummy(k)
//                 } else if k == p.index {
//                     Item::Found
//                 } else {
//                     unreachable!("dummy index > entry")
//                 }
//             }
//             (Some(k), None) => Item::Dummy(cast::<u64, K>(k)),
//             (None, Some(_)) => Item::Found,
//             (None, None) => Item::Empty,
//         };

//         match item {
//             Item::Dummy(k) => Some(Entry::new(k, Cow::Owned(V::empty()))),
//             Item::Found => self.value.next(),
//             Item::Empty => None,
//         }
//     }
// }

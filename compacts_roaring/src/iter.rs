pub use self::{merge::MergeBy, pad::PadUsingDefault, tuples::Tuples};

mod merge;
mod pad;
mod tuples;

//pub struct NotIntoIter<I: Iterator, A> {
//    pad: PadUsingDefault<I, A>,
//}

//impl<I, T> IntoIterator for Not<I>
//where
//    I: IntoIterator<Item = T>,
//    NotIntoIter<I::IntoIter, T>: Iterator<Item = T>,
//{
//    type Item = T;
//    type IntoIter = NotIntoIter<I::IntoIter, T>;
//    fn into_iter(self) -> Self::IntoIter {
//        // FIXME
//        let range = 0..(bits::MAX_BITS / bits::SHORT_BIT_MAX);
//        NotIntoIter {
//            pad: PadUsingDefault::pad_using_default(range, self.val.into_iter()),
//        }
//    }
//}

//impl<'a, I, K: UnsignedInt> Iterator for NotIntoIter<I, Page<K, Cow<'a, Array>>>
//where
//    I: Iterator<Item = Page<K, Cow<'a, Array>>>,
//{
//    type Item = Page<K, Cow<'a, Array>>;
//    fn next(&mut self) -> Option<Self::Item> {
//        self.pad.next().map(|e| {
//            let index = e.index;
//            let value = Array(!&e.value.0);
//            Page::new(index, Cow::Owned(value))
//        })
//    }
//}

//impl<'a, I, K: UnsignedInt> Iterator for NotIntoIter<I, Page<K, Cow<'a, RoaringBlock>>>
//where
//    I: Iterator<Item = Page<K, Cow<'a, RoaringBlock>>>,
//{
//    type Item = Page<K, Cow<'a, RoaringBlock>>;
//    fn next(&mut self) -> Option<Self::Item> {
//        self.pad.next().map(|e| {
//            let index = e.index;
//            let value = RoaringBlock(!&e.value.0);
//            Page::new(index, Cow::Owned(value))
//        })
//    }
//}

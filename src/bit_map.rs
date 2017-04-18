use std::collections::BTreeMap;

use super::{Bits, Bucket};

pub struct BitMap {
    bitmap: BTreeMap<u16, Bucket>,
}

//impl Bits for BitMap {
//    const SIZE: u64 = (<Bucket as Bits>::SIZE) << 16;
//
//    fn none() -> Self {
//        BitMap { bitmap: BTreeMap::new() }
//    }
//    fn ones(&self) -> usize {
//        0
//    }
//}

use std::collections::BTreeMap;

use super::{Bits, Repr};

struct BitMap {
    bitmap: BTreeMap<u16, Repr>,
}

//impl Bits for BitMap {
//    const SIZE: u64 = (<Repr as Bits>::SIZE) << 16;
//
//    fn none() -> Self {
//        BitMap { bitmap: BTreeMap::new() }
//    }
//    fn ones(&self) -> usize {
//        0
//    }
//}

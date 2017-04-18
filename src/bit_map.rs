use std::collections::BTreeMap;

use super::{Bits, Bounded, PopCount, Bucket};

pub struct BitMap {
    pop: PopCount<u32>,
    map: BTreeMap<u16, Bucket>,
}

impl Bits for BitMap {
    const SIZE: u64 = Bucket::SIZE << 16; // 1 << 32

    fn ones(&self) -> u64 {
        self.pop.cardinality()
    }
}

impl BitMap {
    fn new() -> Self {
        BitMap {
            pop: PopCount::MIN,
            map: BTreeMap::new(),
        }
    }
    fn insert(x: u32) -> bool {
        unimplemented!();
    }
}

extern crate cwt;

use std::u32;

use self::cwt::{Bits, BitMap};

#[test]
#[ignore]
fn set_bits_10000000() {
    let mut bits = BitMap::new();
    let mut i = 0;
    while i < 10_000_000 {
        assert!(bits.insert(i));
        i += 1;
    }

    assert_eq!(bits.ones(), 10_000_000);
}

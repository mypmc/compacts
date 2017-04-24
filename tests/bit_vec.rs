extern crate cwt;

use std::u32;

use self::cwt::{PopCount, BitVec};

#[test]
#[ignore]
fn set_bits_10000000() {
    let mut bits = BitVec::new();
    let mut i = 0;
    while i < 10_000_000 {
        assert!(bits.insert(i));
        i += 1;
    }

    assert_eq!(bits.ones(), 10_000_000);
}

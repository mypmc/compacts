#![feature(inclusive_range_syntax)]

extern crate compacts;

use compacts::bits;
use self::bits::pair::*;

fn main() {
    use bits::{Rank, Select0, Select1};

    let mut vec1 = bits::Map64::new();
    let bit = 1 << 60;
    assert!(vec1.insert(bit));
    assert!(vec1.contains(bit));
    assert!(vec1.remove(bit));

    println!("{:#?}", vec1);

    assert_eq!(vec1.rank1(1), 0);
    assert_eq!(vec1.rank0(1), 2);
    assert_eq!(vec1.select1(0), None);

    let max = 100_000;

    for i in 0..max {
        assert_eq!(vec1.select0(i), Some(i));
    }

    let mut vec2 = bits::Map64::new();
    for i in 0..max {
        vec2.insert(i);
    }

    vec1.union_with(&vec2);
    assert_eq!(vec1.count_ones() as u64, max);

    for i in (::std::u64::MAX - max)...::std::u64::MAX {
        vec1.insert(i);
    }

    vec1.optimize();
    vec2.optimize();
    println!("{:#?}\n{:#?}", vec1.summary(), vec2.summary());
}

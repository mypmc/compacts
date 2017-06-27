#![feature(inclusive_range_syntax)]

extern crate compacts;

use compacts::{bits, dict};

fn main() {
    use bits::UnionWith;
    use dict::BitDict;

    let mut vec1 = bits::Vec64::new();
    let bit = 1 << 60;
    assert!(vec1.insert(bit));
    assert!(vec1.contains(bit));
    assert!(vec1.remove(bit));

    println!("{:#?}", vec1);

    assert_eq!(vec1.rank1(1), 0);
    assert_eq!(vec1.rank0(1), 2);
    assert_eq!(vec1.select1(0), None);

    for i in 0..100000 {
        assert_eq!(vec1.select0(i), Some(i));
    }

    let mut vec2 = bits::Vec64::new();
    for i in 0..100000 {
        vec2.insert(i);
    }

    vec1.union_with(&vec2);
    assert_eq!(vec1.count_ones(), 100000);

    for i in (::std::u64::MAX - 100000)...::std::u64::MAX {
        vec1.insert(i);
    }

    vec1.optimize();
    vec2.optimize();
    println!("{:#?}\n{:#?}", vec1.summary(), vec2.summary());
}

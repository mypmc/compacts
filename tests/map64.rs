extern crate compacts;
use compacts::bits::*;
use compacts::dict::PopCount;

#[test]
fn intersection() {
    let map1 = Map64::from(vec![1 << 10, 1 << 20, 1 << 30, 1 << 40, 1 << 50]);
    let map2 = Map64::from(vec![1 << 40, 1 << 50, 1 << 60]);
    let map3 = map1.intersection(&map2);

    assert_eq!(2, map3.count1());
    assert!(map3.contains(1 << 40));
    assert!(map3.contains(1 << 50));
}

#[test]
fn union() {
    let map1 = Map64::from(vec![1 << 1, 1 << 2, 1 << 4]);
    let map2 = Map64::from(vec![1 << 8, 1 << 16, 1 << 32, 1 << 60]);
    let map3 = map1.union(&map2);

    assert_eq!(7, map3.count1());
    assert!(map3.contains(1 << 1));
    assert!(map3.contains(1 << 2));
    assert!(map3.contains(1 << 4));
    assert!(map3.contains(1 << 8));
    assert!(map3.contains(1 << 16));
    assert!(map3.contains(1 << 32));
    assert!(map3.contains(1 << 60));
}

#[test]
fn difference() {
    let map1 = Map64::from(vec![1 << 1, 1 << 2, 1 << 4, 1 << 8, 1 << 16, 1 << 32]);
    let map2 = Map64::from(vec![1 << 8, 1 << 16, 1 << 32]);
    let map3 = map1.difference(&map2);

    assert_eq!(3, map3.count1());
    assert!(map3.contains(1 << 1));
    assert!(map3.contains(1 << 2));
    assert!(map3.contains(1 << 4));
}

#[test]
fn symmetric_difference() {
    let map1 = Map64::from(vec![1 << 10, 1 << 20, 1 << 30, 1 << 40, 1 << 50]);
    let map2 = Map64::from(vec![1 << 40, 1 << 50, 1 << 60]);
    let map3 = map1.symmetric_difference(&map2);

    assert_eq!(4, map3.count1());
    assert!(map3.contains(1 << 10));
    assert!(map3.contains(1 << 20));
    assert!(map3.contains(1 << 30));
    assert!(!map3.contains(1 << 40));
    assert!(!map3.contains(1 << 50));
    assert!(map3.contains(1 << 60));
}

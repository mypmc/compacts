extern crate compacts;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate rand;

#[macro_use]
mod helper;

use std::iter::FromIterator;
use compacts::bits::*;
use compacts::dict::*;

#[test]
fn intersection() {
    let _ = env_logger::init();
    let map1 = Map64::from(vec![1 << 10, 1 << 20, 1 << 30, 1 << 40, 1 << 50]);
    let map2 = Map64::from(vec![1 << 40, 1 << 50, 1 << 60]);

    let mut map = map1.intersection(&map2);
    assert_eq!(2, map.count_ones());
    assert!(map.contains(1 << 40));
    assert!(map.contains(1 << 50));
    map.optimize();
    debug!("{:#?}", map);
}

#[test]
fn union() {
    let _ = env_logger::init();
    let map1 = Map64::from(vec![1 << 1, 1 << 2, 1 << 4]);
    let map2 = Map64::from(vec![1 << 8, 1 << 16, 1 << 32, 1 << 60]);

    let mut map = map1.union(&map2);

    assert_eq!(7, map.count_ones());
    assert!(map.contains(1 << 1));
    assert!(map.contains(1 << 2));
    assert!(map.contains(1 << 4));
    assert!(map.contains(1 << 8));
    assert!(map.contains(1 << 16));
    assert!(map.contains(1 << 32));
    assert!(map.contains(1 << 60));
    map.optimize();
    debug!("{:#?}", map);
}

#[test]
fn difference() {
    let _ = env_logger::init();

    let map1 = Map64::from(vec![1 << 1, 1 << 2, 1 << 4, 1 << 8, 1 << 16, 1 << 32]);
    let map2 = Map64::from(vec![1 << 8, 1 << 16, 1 << 32]);

    let mut map = map1.difference(&map2);

    assert_eq!(3, map.count_ones());
    assert!(map.contains(1 << 1));
    assert!(map.contains(1 << 2));
    assert!(map.contains(1 << 4));
    map.optimize();
    debug!("{:#?}", map);
}

#[test]
fn rank() {
    let _ = env_logger::init();
    let vec = Map64::from(vec![0, 1, 4, 1 << 32, 1 << 50, 1 << 60]);

    assert_eq!(vec.rank0(0), 0);
    assert_eq!(vec.rank0(1), 0);
    assert_eq!(vec.rank0(2), 0);
    assert_eq!(vec.rank0(3), 1);
    assert_eq!(vec.rank0(4), 2);

    assert_eq!(vec.rank1(0), 0);
    assert_eq!(vec.rank1(1), 1);
    assert_eq!(vec.rank1(2), 2);
    assert_eq!(vec.rank1(3), 2);
    assert_eq!(vec.rank1(4), 2);
    assert_eq!(vec.rank1(5), 3);
}

#[test]
fn select() {
    let _ = env_logger::init();

    let vec = Map64::from(vec![0, 1, 4, 1 << 32, 1 << 50, 1 << 60]);

    assert_eq!(vec.select0(0), Some(2));
    assert_eq!(vec.select0(1), Some(3));
    assert_eq!(vec.select0(2), Some(5));

    assert_eq!(vec.select1(0), Some(0));
    assert_eq!(vec.select1(1), Some(1));
    assert_eq!(vec.select1(2), Some(4));
}

#[test]
fn bitmaps() {
    let mut vec1 = Map64::from_iter(
        (0..6000)
            .chain(1_000_000..1_012_000)
            .chain(3_000_000..3_010_000),
    );
    let vec2 = Map64::from_iter(
        (3000..7000)
            .chain(1_006_000..1_018_000)
            .chain(2_000_000..2_010_000),
    );
    let vec3 = Map64::from_iter(
        (0..3000)
            .chain(1_000_000..1_006_000)
            .chain(6000..7000)
            .chain(1_012_000..1_018_000)
            .chain(2_000_000..2_010_000)
            .chain(3_000_000..3_010_000),
    );

    vec1.symmetric_difference_with(&vec2);

    for b in vec1.iter() {
        assert!(vec3.contains(b));
    }
}

#[test]
fn intersection_associativity() {
    let mut rng = rand::thread_rng();
    let vec1 = &setup!(rng, 100, 100);
    let vec2 = &setup!(rng, 100, 100);
    let vec3 = &setup!(rng, 100, 100);
    let i1 = vec1.intersection(vec2).intersection(vec3);
    let i2 = vec1.intersection(&vec2.intersection(vec3));
    let c1 = i1.iter().collect::<Vec<_>>();
    let c2 = i2.iter().collect::<Vec<_>>();
    assert_eq!(c1, c2);
}

#[test]
fn intersection_commutativity() {
    let mut rng = rand::thread_rng();
    let vec1 = &setup!(rng, 20, 20);
    let vec2 = &setup!(rng, 20, 20);
    let i1 = vec1.intersection(vec2);
    let i2 = vec2.intersection(vec1);
    let c1 = i1.iter().collect::<Vec<_>>();
    let c2 = i2.iter().collect::<Vec<_>>();

    if c1 != c2 {
        println!("");
        println!(
            "v1 {:?} {:#?}",
            vec1.iter().collect::<Vec<_>>(),
            vec1.summary()
        );
        println!(
            "v2 {:?} {:#?}",
            vec2.iter().collect::<Vec<_>>(),
            vec2.summary()
        );
    }

    assert_eq!(c1, c2);
}

#[test]
fn union_associativity() {
    let mut rng = rand::thread_rng();
    let vec1 = &setup!(rng, 20, 10);
    let vec2 = &setup!(rng, 200, 100);
    let vec3 = &setup!(rng, 2000, 1000);
    let i1 = vec1.union(vec2).union(vec3);
    let i2 = vec1.union(&vec2.union(vec3));
    let c1 = i1.iter().collect::<Vec<_>>();
    let c2 = i2.iter().collect::<Vec<_>>();

    if c1 != c2 {
        let u1 = vec1.union(vec2);
        let u2 = vec2.union(vec3);
        println!("");
        println!(
            "v1 {:?} {:#?}",
            vec1.iter().collect::<Vec<_>>(),
            vec1.summary()
        );
        println!(
            "v2 {:?} {:#?}",
            vec2.iter().collect::<Vec<_>>(),
            vec2.summary()
        );
        println!(
            "v3 {:?} {:#?}",
            vec3.iter().collect::<Vec<_>>(),
            vec3.summary()
        );
        println!("(v1|v2) {:?}", u1.iter().collect::<Vec<_>>());
        println!("(v2|v3) {:?}", u2.iter().collect::<Vec<_>>());
    }

    assert_eq!(c1, c2);
}

#[test]
fn union_commutativity() {
    let mut rng = rand::thread_rng();
    let vec1 = &setup!(rng, 10, 10);
    let vec2 = &setup!(rng, 100, 100);
    let i1 = vec1.union(vec2);
    let i2 = vec2.union(vec1);
    let c1 = i1.iter().collect::<Vec<_>>();
    let c2 = i2.iter().collect::<Vec<_>>();
    assert_eq!(c1, c2);
}

#[test]
fn symmetric_difference_associativity() {
    let mut rng = rand::thread_rng();

    let vec1 = &setup!(rng, 10, 10);
    let vec2 = &setup!(rng, 10, 10);
    let vec3 = &setup!(rng, 10, 10);

    let i1 = vec1.symmetric_difference(vec2).symmetric_difference(vec3);
    let i2 = vec1.symmetric_difference(&vec2.symmetric_difference(vec3));

    let c1 = i1.iter().collect::<Vec<_>>();
    let c2 = i2.iter().collect::<Vec<_>>();

    if c1 != c2 {
        let sd1 = vec1.symmetric_difference(vec2);
        let sd2 = vec2.symmetric_difference(vec3);
        println!("");
        println!("v1 {:?}", vec1.iter().collect::<Vec<_>>());
        println!("v2 {:?}", vec2.iter().collect::<Vec<_>>());
        println!("v3 {:?}", vec3.iter().collect::<Vec<_>>());
        println!("(v1^v2) {:?}", sd1.iter().collect::<Vec<_>>());
        println!("(v2^v3) {:?}", sd2.iter().collect::<Vec<_>>());
    }

    assert_eq!(c1, c2);
}

#[test]
fn symmetric_difference_commutativity() {
    let mut rng = rand::thread_rng();

    let vec1 = setup!(rng, 10, 10);
    let vec2 = setup!(rng, 10, 10);

    let i1 = (&vec1).symmetric_difference(&vec2);
    let i2 = (&vec2).symmetric_difference(&vec1);

    let c1 = i1.iter().collect::<Vec<_>>();
    let c2 = i2.iter().collect::<Vec<_>>();

    assert_eq!(c1, c2);
}

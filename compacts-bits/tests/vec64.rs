#[macro_use]
extern crate log;
extern crate env_logger;
extern crate rand;
extern crate compacts_bits;

use std::iter::FromIterator;
use compacts_bits::*;
use self::rand::Rng;

fn random_insert<R>(map: &mut Vec64, rng: &mut R, size: u64, max: u64)
where
    R: Rng,
{
    for _ in 0..rng.gen_range(0, size) {
        map.insert(rng.gen_range(0, max));
    }
    map.optimize();
}

#[test]
fn iterator() {
    let _ = env_logger::init();

    {
        let mut bm = Vec64::new();
        for i in 0..1000000 {
            bm.insert(i);
        }
        for (i, bit) in bm.iter().enumerate() {
            assert_eq!(i as u64, bit);
        }
    }
    {
        let b = ::std::u64::MAX - 5;
        let mut bm = Vec64::new();
        for i in b..(b + 3) {
            bm.insert(i);
        }
        let col = bm.iter().collect::<Vec<u64>>();
        assert_eq!(col, vec![b, b + 1, b + 2]);
        debug!("{:#?}", bm);
    }

}

#[test]
fn mem_size() {
    let _ = env_logger::init();
    let mut rng = rand::thread_rng();
    let mut map = Vec64::new();
    random_insert(&mut map, &mut rng, 1 << 16, 1 << 40);
    info!("mem={:#?}", map.mem_size());
}

#[test]
fn intersection() {
    let _ = env_logger::init();
    let map1 = Vec64::from(vec![1 << 10, 1 << 20, 1 << 30, 1 << 40, 1 << 50]);
    let map2 = Vec64::from(vec![1 << 40, 1 << 50, 1 << 60]);

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
    let map1 = Vec64::from(vec![1 << 1, 1 << 2, 1 << 4]);
    let map2 = Vec64::from(vec![1 << 8, 1 << 16, 1 << 32, 1 << 60]);

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

    let map1 = Vec64::from(vec![1 << 1, 1 << 2, 1 << 4, 1 << 8, 1 << 16, 1 << 32]);
    let map2 = Vec64::from(vec![1 << 8, 1 << 16, 1 << 32]);

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
    let vec = Vec64::from(vec![
        0,
        1 << 4,
        1 << 16,
        1 << 32,
        1 << 50,
        1 << 60,
        ::std::u64::MAX,
    ]);

    assert_eq!(vec.rank0(0), 0);
    assert_eq!(vec.rank1(0), 1);
    assert_eq!(vec.rank0(1), 1);
    assert_eq!(vec.rank1(1), 1);
    assert_eq!(vec.rank0(2), 2);
    assert_eq!(vec.rank1(2), 1);
    assert_eq!(vec.rank0(1 << 4), 15);
    assert_eq!(vec.rank1(1 << 4), 2);
    assert_eq!(vec.rank0(1 << 32), (1 << 32) + 1 - 4);
    assert_eq!(vec.rank1(1 << 32), 4);
    assert_eq!(vec.rank0(1 << 50), (1 << 50) + 1 - 5);
    assert_eq!(vec.rank1(1 << 50), 5);
    assert_eq!(vec.rank0(1 << 60), (1 << 60) + 1 - 6);
    assert_eq!(vec.rank1(1 << 60), 6);
}

#[test]
fn select() {
    let _ = env_logger::init();
    let mut vec = Vec64::from(vec![
        0,
        1 << 2, // 4
        1 << 4, // 16
        1 << 16,
        1 << 32,
        1 << 50,
        1 << 60,
        ::std::u64::MAX,
    ]);

    let bit = 1 << 40;
    assert!(vec.insert(bit));
    assert!(vec.remove(bit));
    assert!(!vec.contains(bit));

    assert_eq!(vec.select0(0), Some(1));
    assert_eq!(vec.select1(0), Some(0));

    assert_eq!(vec.select0(1), Some(2));
    assert_eq!(vec.select1(1), Some(1 << 2));

    assert_eq!(vec.select0(2), Some(3));
    assert_eq!(vec.select1(2), Some(1 << 4));

    assert_eq!(vec.select0(3), Some(5));
    assert_eq!(vec.select1(3), Some(1 << 16));

    assert_eq!(vec.select0(4), Some(6));
    assert_eq!(vec.select1(4), Some(1 << 32));

    assert_eq!(vec.select0(5), Some(7));
    assert_eq!(vec.select1(5), Some(1 << 50));

    assert_eq!(vec.select0(6), Some(8));
    assert_eq!(vec.select1(6), Some(1 << 60));

    assert_eq!(vec.select0(7), Some(9));
    assert_eq!(vec.select1(7), Some(::std::u64::MAX));

    for i in 1..vec.count_ones() {
        let r = vec.rank1(vec.select1(i as u64).unwrap_or(0));
        if r == 0 {
            assert_eq!(i, 0);
        } else {
            assert_eq!(i, r - 1);
        }
    }
}

#[test]
fn bitmaps() {
    let mut vec1 = Vec64::from_iter((0..6000).chain(1000000..1012000).chain(3000000..3010000));
    let vec2 = Vec64::from_iter((3000..7000).chain(1006000..1018000).chain(2000000..2010000));
    let vec3 = Vec64::from_iter(
        (0..3000)
            .chain(1000000..1006000)
            .chain(6000..7000)
            .chain(1012000..1018000)
            .chain(2000000..2010000)
            .chain(3000000..3010000),
    );

    vec1.symmetric_difference_with(&vec2);

    for b in vec1.iter() {
        assert!(vec3.contains(b));
    }
}

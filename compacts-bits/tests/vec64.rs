#[macro_use]
extern crate log;
extern crate env_logger;
extern crate rand;
extern crate compacts_bits;

use self::rand::Rng;

use compacts_bits::Vec64;
use compacts_bits::ops::*;

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

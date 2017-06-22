#[macro_use]
extern crate log;
extern crate env_logger;
extern crate rand;
extern crate compacts_bits;

use self::rand::Rng;

use compacts_bits::BitMap;

fn random_insert<R>(map: &mut BitMap, rng: &mut R, size: u64, max: u64)
    where
    R: Rng,
{
    for _ in 0..rng.gen_range(0, size) {
        map.insert(rng.gen_range(0, max));
    }
}

#[test]
fn iterator() {
    let _ = env_logger::init();

    {
        let mut bm = BitMap::new();
        for i in 0..1000000 {
            bm.insert(i);
        }
        for (i, bit) in bm.iter().enumerate() {
            assert_eq!(i as u64, bit);
        }
    }
    {
        let b = 1 << 50;
        let mut bm = BitMap::new();
        for i in b..(b + 3) {
            bm.insert(i);
            debug!("{:#?}", i);
        }
        let col = bm.iter().collect::<Vec<u64>>();
        assert_eq!(col, vec![b, b+1, b+2]);
        debug!("{:#?}", bm);
    }

}

#[test]
fn mem_size() {
    let _ = env_logger::init();
    let mut rng = rand::thread_rng();
    let mut map = BitMap::new();
    random_insert(&mut map, &mut rng, 1 << 16, 1 << 40);
    map.optimize();
    info!("mem={:#?}", map.mem_size());
}

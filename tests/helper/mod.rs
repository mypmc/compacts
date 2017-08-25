#![allow(warnings)]

use compacts::bits::*;
use rand::Rng;

pub fn random_insert<R>(map: &mut Map64, rng: &mut R, size: u64, max: u64)
where
    R: Rng,
{
    for _ in 0..size {
        map.insert(rng.gen_range(0, max));
    }
    map.optimize();
}

macro_rules! setup {
    ( $rng:expr, $max:expr, $size:expr ) => {
        {
            let mut vec = Map64::new();
            helper::random_insert(&mut vec, &mut $rng, $size, $max);
            vec
        }
    };
}

macro_rules! bit_vec {
    ( ) => {
        &Map32::new()
    };

    ( $size:expr, $end:expr, $rng:expr ) => {
        {
            bit_vec!($size, 0, $end, $rng)
        }
    };

    ( $size:expr, $start:expr, $end:expr, $rng:expr ) => {
        {
            let mut vec = Map32::new();
            for _ in 0..$size {
                let gen = $rng.gen_range($start, $end);
                vec.insert(gen);
            }
            vec.optimize();
            vec
        }
    };
}

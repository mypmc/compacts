use crate::bits::mask::{and, or, xor, BitMask, Fold};

use super::{BitMap, Bytes};

use lazy_static::lazy_static;
// use quickcheck::quickcheck;
use rand::prelude::*;

use std::{fs, io::BufReader};

macro_rules! generate {
    ($rng: expr, $nbits: expr, $bound: expr) => {{
        let mut build = BitMap::default();
        for _ in 0..$nbits {
            build.put1($rng.gen_range(0, $bound));
        }
        build
    }};
}

const BOUND: u64 = 10_000_000;

lazy_static! {
    static ref NBITS: u64 = BOUND / thread_rng().gen_range(1, 100);
    static ref V0: BitMap = generate!(thread_rng(), *NBITS, BOUND);
    static ref V1: BitMap = generate!(thread_rng(), *NBITS, BOUND);
    static ref V2: BitMap = generate!(thread_rng(), *NBITS, BOUND);
}

mod io {
    use super::*;

    #[test]
    fn decode_runs() {
        static PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/compressed_runs");
        let map1 = {
            let rdr = BufReader::new(fs::File::open(PATH).expect("open file"));
            let mut map = BitMap::deserialize_from(rdr).expect("deserialize map");
            map.optimize();
            map
        };

        let map2 = {
            let mut buf = Vec::with_capacity(1 << 11);
            map1.serialize_into(&mut buf).unwrap();
            BitMap::deserialize_from(&buf[..]).unwrap()
        };

        assert_eq!(map1, map2);
    }

    #[test]
    fn decode_noruns() {
        static PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/compressed_noruns");
        let map1 = {
            let rdr = BufReader::new(fs::File::open(PATH).expect("open file"));
            let mut map = BitMap::deserialize_from(rdr).expect("deserialize map");
            map.optimize();
            map
        };

        let map2 = {
            let mut buf = Vec::with_capacity(1 << 11);
            map1.serialize_into(&mut buf).unwrap();
            BitMap::deserialize_from(&buf[..]).unwrap()
        };

        assert_eq!(map1, map2);
    }
}

mod mask {
    use super::*;
    macro_rules! associative {
        ($x: expr,$y: expr,$z: expr,$fn: ident) => {{
            let r1 = $fn($fn($x, $y), $z).into_iter().collect::<BitMap>();
            let r2 = $fn($x, $fn($y, $z)).into_iter().collect::<BitMap>();
            r1 == r2
        }};
    }

    macro_rules! commutative {
        ($x: expr,$y: expr,$fn: ident) => {{
            let r1 = $fn($x, $y).into_iter().collect::<BitMap>();
            let r2 = $fn($y, $x).into_iter().collect::<BitMap>();
            r1 == r2
        }};
    }

    #[test]
    fn associative_and() {
        assert!(associative!(&*V0, &*V1, &*V2, and), "and");
    }
    #[test]
    fn associative_or() {
        assert!(associative!(&*V0, &*V1, &*V2, or), "or");
    }
    #[test]
    fn associative_xor() {
        assert!(associative!(&*V0, &*V1, &*V2, xor), "xor");
    }

    #[test]
    fn commutative_and() {
        assert!(commutative!(&*V0, &*V1, and), "V0 & V1");
        assert!(commutative!(&*V1, &*V2, and), "V1 & V2");
        assert!(commutative!(&*V0, &*V2, and), "V0 & V2");
    }

    #[test]
    fn commutative_or() {
        assert!(commutative!(&*V0, &*V1, or), "V0 | V1");
        assert!(commutative!(&*V1, &*V2, or), "V1 | V2");
        assert!(commutative!(&*V0, &*V2, or), "V0 | V2");
    }

    #[test]
    fn commutative_xor() {
        assert!(commutative!(&*V0, &*V1, xor), "V0 ^ V1");
        assert!(commutative!(&*V1, &*V2, xor), "V1 ^ V2");
        assert!(commutative!(&*V0, &*V2, xor), "V0 ^ V2");
    }
}

mod fold {
    use super::*;
    #[test]
    fn and() {
        let map1 = Fold::and(vec![&*V0, &*V1, &*V2])
            .into_steps()
            .collect::<BitMap>();
        let map2 = V0.and(&*V1).and(&*V2).into_steps().collect::<BitMap>();
        assert_eq!(map1, map2);
    }

    #[test]
    fn or() {
        let map1 = Fold::or(vec![&*V0, &*V1, &*V2])
            .into_steps()
            .collect::<BitMap>();
        let map2 = V0.or(&*V1).or(&*V2).into_steps().collect::<BitMap>();
        assert_eq!(map1, map2);
    }

    #[test]
    fn xor() {
        let map1 = Fold::xor(vec![&*V0, &*V1, &*V2])
            .into_steps()
            .collect::<BitMap>();
        let map2 = V0.xor(&*V1).xor(&*V2).into_steps().collect::<BitMap>();
        assert_eq!(map1, map2);
    }
}

#[test]
fn rank_select() {
    for _ in 0..1000 {
        let rank1 = thread_rng().gen_range(0, V0.count1());
        assert!(V0.rank1(V0.select1(rank1).unwrap()) == rank1);
        let rank0 = thread_rng().gen_range(0, V0.count0());
        assert!(V0.rank0(V0.select0(rank0).unwrap()) == rank0);
    }
}

mod bytes {
    use super::*;
    use std::io::Cursor;

    lazy_static! {
        static ref BUF0: Vec<u8> = {
            let mut vec = Vec::with_capacity(1 << 16);
            V0.serialize_into(&mut vec).unwrap();
            vec
        };
        static ref BUF1: Vec<u8> = {
            let mut vec = Vec::with_capacity(1 << 16);
            V1.serialize_into(&mut vec).unwrap();
            vec
        };
        static ref BUF2: Vec<u8> = {
            let mut vec = Vec::with_capacity(1 << 16);
            V2.serialize_into(&mut vec).unwrap();
            vec
        };
    }

    mod prop {
        use super::*;

        #[test]
        fn identity() {
            let m0 = BitMap::deserialize_from(Cursor::new(&*BUF0)).expect("BUF0");
            let m1 = BitMap::deserialize_from(Cursor::new(&*BUF1)).expect("BUF1");
            let m2 = BitMap::deserialize_from(Cursor::new(&*BUF2)).expect("BUF2");
            let b0 = Bytes::new(&*BUF0).expect("BUF0");
            let b1 = Bytes::new(&*BUF1).expect("BUF1");
            let b2 = Bytes::new(&*BUF2).expect("BUF2");
            assert_eq!(m0, b0.into_steps().collect::<BitMap>());
            assert_eq!(m1, b1.into_steps().collect::<BitMap>());
            assert_eq!(m2, b2.into_steps().collect::<BitMap>());
        }
    }

    mod mask {
        use super::*;

        #[test]
        fn and() {
            let m0 = BitMap::deserialize_from(Cursor::new(&*BUF0)).expect("BUF0");
            let m1 = BitMap::deserialize_from(Cursor::new(&*BUF1)).expect("BUF1");
            let m2 = BitMap::deserialize_from(Cursor::new(&*BUF2)).expect("BUF2");
            let b0 = Bytes::new(&*BUF0).expect("BUF0");
            let b1 = Bytes::new(&*BUF1).expect("BUF1");
            let b2 = Bytes::new(&*BUF2).expect("BUF2");
            let r0 = m0.and(&m1).and(&m2).into_steps().collect::<BitMap>();
            let r1 = b2.and(&b1).and(&b0).into_steps().collect::<BitMap>();
            let r2 = V0.and(&*V1).and(&*V2).into_steps().collect::<BitMap>();
            assert_eq!(r0, r1);
            assert_eq!(r0, r2);
        }

        #[test]
        fn or() {
            let m0 = BitMap::deserialize_from(Cursor::new(&*BUF0)).expect("BUF0");
            let m1 = BitMap::deserialize_from(Cursor::new(&*BUF1)).expect("BUF1");
            let m2 = BitMap::deserialize_from(Cursor::new(&*BUF2)).expect("BUF2");
            let b0 = Bytes::new(&*BUF0).expect("BUF0");
            let b1 = Bytes::new(&*BUF1).expect("BUF1");
            let b2 = Bytes::new(&*BUF2).expect("BUF2");
            let r0 = m0.or(&m1).or(&m2).into_steps().collect::<BitMap>();
            let r1 = b2.or(&b1).or(&b0).into_steps().collect::<BitMap>();
            let r2 = V0.or(&*V1).or(&*V2).into_steps().collect::<BitMap>();
            assert_eq!(r0, r1);
            assert_eq!(r0, r2);
        }

        #[test]
        fn xor() {
            let m0 = BitMap::deserialize_from(Cursor::new(&*BUF0)).expect("BUF0");
            let m1 = BitMap::deserialize_from(Cursor::new(&*BUF1)).expect("BUF1");
            let m2 = BitMap::deserialize_from(Cursor::new(&*BUF2)).expect("BUF2");
            let b0 = Bytes::new(&*BUF0).expect("BUF0");
            let b1 = Bytes::new(&*BUF1).expect("BUF1");
            let b2 = Bytes::new(&*BUF2).expect("BUF2");
            let r0 = m0.xor(&m1).xor(&m2).into_steps().collect::<BitMap>();
            let r1 = b2.xor(&b1).xor(&b0).into_steps().collect::<BitMap>();
            let r2 = V0.xor(&*V1).xor(&*V2).into_steps().collect::<BitMap>();
            assert_eq!(r0, r1);
            assert_eq!(r0, r2);
        }
    }
}

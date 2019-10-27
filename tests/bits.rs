#[allow(unused_imports)]
use {
    compacts::{
        bits::{and, or, xor, Fold, Mask},
        ops::*,
        BitArray, BitMap, WaveletMatrix,
    },
    lazy_static::lazy_static,
    rand::prelude::*,
};

macro_rules! generate {
    (Vec; $rng:expr, $nbits:expr, $bound:expr) => {{
        let mut build = vec![0; compacts::bits::blocks_by($bound, 64)];
        for _ in 0..$nbits {
            build.put1($rng.gen_range(0, $bound));
        }
        build
    }};
    (BitMap; $rng:expr, $nbits:expr, $bound:expr) => {{
        let mut build = BitMap::none($bound);
        dbg!(build.size());
        for _ in 0..$nbits {
            build.put1($rng.gen_range(0, $bound - 1));
        }
        build
    }};
}

const BOUND: usize = 10_000_000;

lazy_static! {
    static ref NBITS: usize = BOUND / thread_rng().gen_range(1, 100);

    static ref V0: Vec<u64> = generate!(Vec; thread_rng(), *NBITS, BOUND);
    static ref V1: Vec<u64> = generate!(Vec; thread_rng(), *NBITS, BOUND);
    static ref V2: Vec<u64> = generate!(Vec; thread_rng(), *NBITS, BOUND);

    static ref A0: BitArray<u64> = BitArray::from(V0.clone());
    static ref A1: BitArray<u64> = BitArray::from(V1.clone());
    static ref A2: BitArray<u64> = BitArray::from(V2.clone());

    // static ref M0: BitMap<[u64; 1024]> = generate!(BitMap; thread_rng(), *NBITS, BOUND);
    // static ref M1: BitMap<[u64; 1024]> = generate!(BitMap; thread_rng(), *NBITS, BOUND);
    // static ref M2: BitMap<[u64; 1024]> = generate!(BitMap; thread_rng(), *NBITS, BOUND);

    static ref M0: BitMap<[u64; 512]> = generate!(BitMap; thread_rng(), *NBITS, BOUND);
    static ref M1: BitMap<[u64; 512]> = generate!(BitMap; thread_rng(), *NBITS, BOUND);
    static ref M2: BitMap<[u64; 512]> = generate!(BitMap; thread_rng(), *NBITS, BOUND);
}

mod mask {
    use super::*;

    macro_rules! associative {
        ($x:expr, $y:expr, $z:expr, $fn:ident) => {{
            let mut vec = vec![$x, $y, $z];
            let mut rng = thread_rng();
            let r1 = {
                SliceRandom::shuffle(&mut vec[..], &mut rng);
                Fold::$fn(vec.clone()).collect::<Vec<_>>()
            };

            let r2 = {
                SliceRandom::shuffle(&mut vec[..], &mut rng);
                Fold::$fn(vec.clone()).collect::<Vec<_>>()
            };

            r1 == r2
        }};
    }

    macro_rules! commutative {
        ($x:expr, $y:expr, $fn:ident) => {{
            let r1 = $fn($x, $y).collect::<Vec<_>>();
            let r2 = $fn($y, $x).collect::<Vec<_>>();
            r1 == r2
        }};
    }

    #[test]
    fn associative_and() {
        assert!(associative!(&*M0, &*M1, &*M2, and), "and");
    }

    #[test]
    fn associative_or() {
        assert!(associative!(&*M0, &*M1, &*M2, or), "or");
    }

    #[test]
    fn associative_xor() {
        assert!(associative!(&*M0, &*M1, &*M2, xor), "xor");
    }

    #[test]
    fn commutative_and() {
        assert!(commutative!(&*M0, &*M1, and), "M0 & M1");
        assert!(commutative!(&*M1, &*M2, and), "M1 & M2");
        assert!(commutative!(&*M0, &*M2, and), "M0 & M2");
    }

    #[test]
    fn commutative_or() {
        assert!(commutative!(&*M0, &*M1, or), "M0 | M1");
        assert!(commutative!(&*M1, &*M2, or), "M1 | M2");
        assert!(commutative!(&*M0, &*M2, or), "M0 | M2");
    }

    #[test]
    fn commutative_xor() {
        assert!(commutative!(&*M0, &*M1, xor), "M0 ^ M1");
        assert!(commutative!(&*M1, &*M2, xor), "M1 ^ M2");
        assert!(commutative!(&*M0, &*M2, xor), "M0 ^ M2");
    }

    #[test]
    fn fold_mask() {
        let data = vec![&*M0, &*M1, &*M2];
        let vec1 = Fold::and(data)
            .or(&*M0)
            .or(&*M1)
            .or(&*M2)
            .collect::<Vec<_>>();
        let vec2 = M0.or(&*M1).or(&*M2).collect::<Vec<_>>();
        assert_eq!(vec1, vec2);
    }

    #[test]
    fn fold_and() {
        let vec1 = Fold::and(vec![&*M0, &*M1, &*M2]).collect::<Vec<_>>();
        let vec2 = M0.and(&*M1).and(&*M2).collect::<Vec<_>>();
        assert_eq!(vec1, vec2);
    }

    #[test]
    fn fold_or() {
        let vec1 = Fold::or(vec![&*M0, &*M1, &*M2]).collect::<Vec<_>>();
        let vec2 = M0.or(&*M1).or(&*M2).collect::<Vec<_>>();
        assert_eq!(vec1, vec2);
    }

    #[test]
    fn fold_xor() {
        let vec1 = Fold::xor(vec![&*M0, &*M1, &*M2]).collect::<Vec<_>>();
        let vec2 = M0.xor(&*M1).xor(&*M2).collect::<Vec<_>>();
        assert_eq!(vec1, vec2);
    }
}

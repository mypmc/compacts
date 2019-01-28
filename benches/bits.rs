#![feature(test)]

extern crate compacts;
extern crate test;
#[macro_use]
extern crate lazy_static;
extern crate rand;

macro_rules! bench {
    ($Repr:ty, $NBITS:expr, $BOUND:expr) => {
        use compacts::bits::{self, *};
        use rand::prelude::*;
        use test::Bencher;

        type Type = PageMap<u64, $Repr>;

        macro_rules! bits {
            ($rng: expr) => {{
                let mut bits = Type::new();
                for _ in 0..$NBITS {
                    bits.set1($rng.gen_range(0, $BOUND));
                }
                bits
            }};
        }

        lazy_static! {
            static ref XS: Vec<Type> = vec![
                bits!(rng()),
                bits!(rng()),
                bits!(rng()),
                bits!(rng()),
                bits!(rng()),
            ];
            static ref V0: &'static Type = &XS[0];
            static ref V1: &'static Type = &XS[1];
            static ref V2: &'static Type = &XS[2];
            static ref V3: &'static Type = &XS[3];
            static ref V4: &'static Type = &XS[4];
        }

        fn rng() -> ThreadRng {
            rand::thread_rng()
        }

        #[bench]
        fn access(bench: &mut Bencher) {
            let bits = &*V0;
            bench.iter(|| bits.access(std::cmp::min(1 << 32, rng().gen())));
        }

        #[bench]
        fn rank1(bench: &mut Bencher) {
            let bits = &*V0;
            bench.iter(|| bits.rank1(rng().gen_range(0, bits.bits())));
        }

        #[bench]
        fn select1(bench: &mut Bencher) {
            let bits = &*V0;
            bench.iter(|| bits.rank1(rng().gen_range(0, bits.bits() - 1)));
        }

        #[bench]
        fn set1(bench: &mut Bencher) {
            let mut bits = V0.clone();
            bench.iter(|| bits.set1(std::cmp::min(1 << 32, rng().gen())));
        }

        #[bench]
        fn set0(bench: &mut Bencher) {
            let mut bits = V0.clone();
            bench.iter(|| bits.set0(std::cmp::min(1 << 32, rng().gen())));
        }

        #[bench]
        fn and(bench: &mut Bencher) {
            bench.iter(|| V1.and(*V2).into_iter().collect::<Type>());
        }

        // #[bench]
        // fn and_not(bench: &mut Bencher) {
        //     bench.iter(|| V1.and(V2.not()).into_iter().collect::<Type>());
        // }

        #[bench]
        fn or(bench: &mut Bencher) {
            bench.iter(|| V1.or(*V2).into_iter().collect::<Type>());
        }

        #[bench]
        fn xor(bench: &mut Bencher) {
            bench.iter(|| V1.xor(*V2).into_iter().collect::<Type>());
        }

        #[bench]
        fn fold_and(bench: &mut Bencher) {
            bench.iter(|| bits::Fold::and(&*XS).into_iter().collect::<Type>());
        }

        #[bench]
        fn fold_or(bench: &mut Bencher) {
            bench.iter(|| bits::Fold::or(&*XS).into_iter().collect::<Type>());
        }

        #[bench]
        fn fold_xor(bench: &mut Bencher) {
            bench.iter(|| bits::Fold::xor(&*XS).into_iter().collect::<Type>());
        }
    };
}

macro_rules! genmod {
    ($BITSIZE:expr) => {
        // mod array_00 {
        //     bench!(Array, $BITSIZE / 1000, $BITSIZE);
        // }
        // mod array_01 {
        //     bench!(Array, $BITSIZE / 100, $BITSIZE);
        // }
        // mod array_05 {
        //     bench!(Array, $BITSIZE / 20, $BITSIZE);
        // }
        // mod array_10 {
        //     bench!(Array, $BITSIZE / 10, $BITSIZE);
        // }
        // mod array_50 {
        //     bench!(Array, $BITSIZE / 2, $BITSIZE);
        // }

        mod block_00 {
            bench!(Block<u64>, $BITSIZE / 1000, $BITSIZE);
        }
        mod block_01 {
            bench!(Block<u64>, $BITSIZE / 100, $BITSIZE);
        }
        mod block_05 {
            bench!(Block<u64>, $BITSIZE / 20, $BITSIZE);
        }
        mod block_10 {
            bench!(Block<u64>, $BITSIZE / 10, $BITSIZE);
        }
        mod block_50 {
            bench!(Block<u64>, $BITSIZE / 2, $BITSIZE);
        }
    };
}

mod size_001m {
    genmod!(1_000_000);
}

mod size_005m {
    genmod!(5_000_000);
}

mod size_010m {
    genmod!(10_000_000);
}

mod size_100m {
    genmod!(100_000_000);
}

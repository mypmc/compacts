use crate::bit::rrr::Code;
use crate::bit::*;

#[test]
fn test_read_code() {
    let slice: &[u8] = &[
        0b_1100_1010_u8,
        0b_0010_1101,
        0b_0010_1011,
        0b_0010_1011,
        0b_0010_1011,
    ];

    let mut read: u8;

    assert_eq!(Read::<u8>::read(slice, 0..7), 0b_01001010_u8);

    read = slice.read(0..7);
    assert_eq!(read, 0b_01001010_u8);
    read = slice.read(3..4);
    assert_eq!(read, 0b_00000001_u8);
    read = slice.read(3..7);
    assert_eq!(read, 0b_00001001_u8);
    read = slice.read(7..10);
    assert_eq!(read, 0b_00000011_u8);
    read = slice.read(13..20);
    assert_eq!(read, 0b_01011001_u8);

    assert_eq!(
        Read::<u16>::read(slice, 0..15),
        0b_0000_00101101_11001010_u16
    );

    assert_eq!(
        Read::<u32>::read(slice, 8..27),
        0b_0011_00101011_00101101_u32
    );
    assert_eq!(
        Read::<u64>::read(slice, 8..27),
        0b_0011_00101011_00101101_u64
    );
}

#[test]
fn default_value() {
    let zero = <u64 as UnsignedInt>::ZERO;
    assert_eq!(zero, <u64 as Default>::default());
}

quickcheck! {
    fn update_all(vec1: Vec<u64>, vec2: Vec<u64>) -> bool {
        let mut v1 = vec1;
        let mut v2 = vec2;

        let c1 = v1.count1();
        let r1 = c1 == v1.set0(..);

        let c0 = v2.count0();
        let r2 = c0 == v2.set1(..);

        r1 && r2
    }
}

quickcheck! {
    fn rrr8(code: u8) -> bool {
        let (class, offset) = rrr::encode(code);
        let got = rrr::decode(class, offset);
        got == code & u8::MASK
    }

    fn rrr16(code: u16) -> bool {
        let (class, offset) = rrr::encode(code);
        let got = rrr::decode(class, offset);
        got == code & u16::MASK
    }

    fn rrr32(code: u32) -> bool {
        let (class, offset) = rrr::encode(code);
        let got = rrr::decode(class, offset);
        got == code & u32::MASK
    }

    fn rrr64(code: u64) -> bool {
        let (class, offset) = rrr::encode(code);
        let got = rrr::decode(class, offset);
        got == code & u64::MASK
    }

    fn rrrsize(code: usize) -> bool {
        let (class, offset) = rrr::encode(code);
        let got = rrr::decode(class, offset);
        got == code & usize::MASK
    }

}

quickcheck! {
    fn rank_select(vec: Vec<u64>) -> bool {
        let mut bytes = Map::<u8>::new();
        for &i in &vec {
            bytes.set1(i);
        }

        let mut entries = KeyMap::<usize, u8>::new();
        for &i in &vec {
            entries.set1(i);
        }

        let a = (0..bytes.count1()).take(1000).all(|i| {
            bytes.rank1(bytes.select1(i).unwrap()) == i
        });
        let b = (0..bytes.count0()).take(1000).all(|i| {
            bytes.rank0(bytes.select0(i).unwrap()) == i
        });

        let c = (0..entries.count1()).take(1000).all(|i| {
            entries.rank1(entries.select1(i).unwrap()) == i
        });
        let d = (0..entries.count0()).take(1000).all(|i| {
            entries.rank0(entries.select0(i).unwrap()) == i
        });

        a && b && c && d
    }

    fn search_select(vec: Vec<u64>) -> bool {
        (0..vec.count1()).all(|i| {
            vec.search1(i) == vec.select1(i)
        })
    }
}

macro_rules! gen {
    ($Type:ty, $NBITS:expr, $BOUND:expr) => {
        use super::*;
        use crate::bit::*;
        use rand::prelude::*;

        type Type = Map<$Type>;

        macro_rules! associative {
            ($x: expr,$y: expr,$z: expr,$fn: ident) => {{
                let r1 = $fn($fn($x, $y), $z).into_iter().collect::<Type>();
                let r2 = $fn($x, $fn($y, $z)).into_iter().collect::<Type>();
                r1 == r2
            }};
        }

        macro_rules! commutative {
            ($x: expr,$y: expr,$fn: ident) => {{
                let r1 = $fn($x, $y).into_iter().collect::<Type>();
                let r2 = $fn($y, $x).into_iter().collect::<Type>();
                r1 == r2
            }};
        }

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
            static ref V0: Type = bits!(rng());
            static ref V1: Type = bits!(rng());
            static ref V2: Type = bits!(rng());
        }

        fn rng() -> ThreadRng {
            rand::thread_rng()
        }

        #[test]
        fn associative() {
            assert!(associative!(&*V0, &*V1, &*V2, and));
            assert!(associative!(&*V0, &*V1, &*V2, or));
            assert!(associative!(&*V0, &*V1, &*V2, xor));
        }

        #[test]
        fn commutative() {
            assert!(commutative!(&*V0, &*V1, and));
            assert!(commutative!(&*V1, &*V2, and));
            assert!(commutative!(&*V0, &*V2, and));

            assert!(commutative!(&*V0, &*V1, or));
            assert!(commutative!(&*V1, &*V2, or));
            assert!(commutative!(&*V0, &*V2, or));

            assert!(commutative!(&*V0, &*V1, xor));
            assert!(commutative!(&*V1, &*V2, xor));
            assert!(commutative!(&*V0, &*V2, xor));
        }
    };
}

mod block {
    const BITSIZE: u64 = 010_000_000;

    mod density_00 {
        gen!(Block<[u64; 1024]>, BITSIZE / 1000, BITSIZE);
    }
    mod density_05 {
        gen!(Block<[u64; 1024]>, BITSIZE / 20, BITSIZE);
    }
    mod density_10 {
        gen!(Block<[u64; 1024]>, BITSIZE / 10, BITSIZE);
    }
    mod density_20 {
        gen!(Block<[u64; 1024]>, BITSIZE / 5, BITSIZE);
    }
    mod density_50 {
        gen!(Block<[u64; 1024]>, BITSIZE / 2, BITSIZE);
    }
}

mod entry {
    const BITSIZE: u64 = 010_000_000;

    mod density_00 {
        gen!(Entry<u64, Block<[u64; 1024]>>, BITSIZE/1000, BITSIZE);
    }
    mod density_05 {
        gen!(Entry<u64, Block<[u64; 1024]>>, BITSIZE/20, BITSIZE);
    }
    mod density_10 {
        gen!(Entry<u64, Block<[u64; 1024]>>, BITSIZE/10, BITSIZE);
    }
    mod density_20 {
        gen!(Entry<u64, Block<[u64; 1024]>>, BITSIZE/5, BITSIZE);
    }
    mod density_50 {
        gen!(Entry<u64, Block<[u64; 1024]>>, BITSIZE/2, BITSIZE);
    }
}

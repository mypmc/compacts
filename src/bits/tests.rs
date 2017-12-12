use std::{io, u16};
use quickcheck::TestResult;

use io::{ReadFrom, WriteTo};
use bits::*;

fn to_seq16(vec: &Vec<u16>) -> Seq16 {
    Seq16::from(vec.clone())
}
fn to_arr64(vec: &Vec<u16>) -> Arr64 {
    vec.iter().collect()
}
fn to_rle16(vec: &Vec<u16>) -> Run16 {
    vec.iter().collect()
}

fn all_kind_block(vec: &Vec<u16>) -> (Repr, Repr, Repr) {
    let b1 = Repr::Seq(to_seq16(vec));
    let b2 = Repr::Arr(to_arr64(vec));
    let b3 = Repr::Run(to_rle16(vec));
    (b1, b2, b3)
}

quickcheck!{
    fn prop_u64_split_merge_identity(w: u64) -> bool {
        w == <u64 as Merge>::merge(w.split())
    }
    fn prop_u64_pop_count(word: u64) -> TestResult {
        let c1: u32 = word.count1();
        let c0: u32 = word.count0();
        TestResult::from_bool(c1 + c0 == <u64 as PopCount<u32>>::SIZE)
    }
    fn prop_u64_rank0_rank1(word: u64, i: u32) -> TestResult {
        TestResult::from_bool(word.rank1(i) + word.rank0(i) == i)
    }
    fn prop_u64_rank0_select0(word: u64, i: u32) -> bool {
        if let Some(p) = word.select0(i) {
            return word.rank0(p) == i;
        }
        true
    }
    fn prop_u64_rank1_select1(word: u64, i: u32) -> bool {
        if let Some(p) = word.select1(i) {
            return word.rank1(p) == i;
        }
        true
    }
}

macro_rules! check_rank_select {
    ( 0, $block:expr, $i:expr ) => {
        if let Some(p) = $block.select0($i) {
            return $block.rank0(p) == $i;
        }
    };
    ( 1, $block:expr, $i:expr ) => {
        if let Some(p) = $block.select1($i) {
            return $block.rank1(p) == $i;
        }
    }
}

quickcheck!{
    fn prop_block_all_count1(vec: Vec<u16>) -> TestResult {
        let (b1, b2, b3) = all_kind_block(&vec);
        TestResult::from_bool(
            b1.count1() == b2.count1() && b2.count1() == b3.count1()
        )
    }

    fn prop_block_all_count0(vec: Vec<u16>) -> TestResult {
        let (b1, b2, b3) = all_kind_block(&vec);
        TestResult::from_bool(
            b1.count0() == b2.count0() && b2.count0() == b3.count0()
        )
    }

    fn prop_block_all_rank0(vec: Vec<u16>, i: u16) -> TestResult {
        let (b1, b2, b3) = all_kind_block(&vec);
        TestResult::from_bool(b1.rank0(i) == b2.rank0(i) && b2.rank0(i) == b3.rank0(i))
    }

    fn prop_block_all_rank1(vec: Vec<u16>, i: u16) -> TestResult {
        let (b1, b2, b3) = all_kind_block(&vec);
        TestResult::from_bool(b1.rank1(i) == b2.rank1(i) && b2.rank1(i) == b3.rank1(i))
    }

    fn prop_block_all_select0(vec: Vec<u16>, i: u16) -> TestResult {
        let (b1, b2, b3) = all_kind_block(&vec);
        TestResult::from_bool(
            b1.select0(i) == b2.select0(i) && b2.select0(i) == b3.select0(i)
        )
    }

    fn prop_block_all_select1(vec: Vec<u16>, i: u16) -> TestResult {
        let (b1, b2, b3) = all_kind_block(&vec);
        TestResult::from_bool(
            b1.select1(i) == b2.select1(i) && b2.select1(i) == b3.select1(i)
        )
    }
}

quickcheck!{
    fn prop_set_rank0_rank1(vec: Vec<u32>, i: u32) -> TestResult {
        let b = Set::from(vec);
        TestResult::from_bool(b.rank1(i) + b.rank0(i) == i)
    }
    fn prop_set_rank0_select0(vec: Vec<u32>, i: u32) -> bool {
        let b = Set::from(vec);
        check_rank_select!(0, b, i);
        true
    }
    fn prop_set_rank1_select1(vec: Vec<u32>, i: u32) -> bool {
        let b = Set::from(vec);
        check_rank_select!(1, b, i);
        true
    }
}

quickcheck!{
    fn prop_block_seq16_rank0_rank1(vec: Vec<u16>, i: u16) -> TestResult {
        let b = Repr::Seq(to_seq16(&vec));
        TestResult::from_bool(b.rank1(i) + b.rank0(i) == i)
    }

    fn prop_block_seq16_rank0_select0(vec: Vec<u16>, i: u16) -> bool {
        let b = Repr::Seq(to_seq16(&vec));
        check_rank_select!(0, b, i);
        true
    }

    fn prop_block_seq16_rank1_select1(vec: Vec<u16>, i: u16) -> bool {
        let b = Repr::Seq(to_seq16(&vec));
        check_rank_select!(1, b, i);
        true
    }
}

quickcheck!{
    fn prop_block_arr64_rank0_rank1(vec: Vec<u16>, i: u16) -> TestResult {
        let b = Repr::Arr(to_arr64(&vec));
        TestResult::from_bool(b.rank1(i) + b.rank0(i) == i)
    }
    fn prop_block_arr64_rank0_select0(vec: Vec<u16>, i: u16) -> bool {
        let b = Repr::Arr(to_arr64(&vec));
        check_rank_select!(0, b, i);
        true
    }
    fn prop_block_arr64_rank1_select1(vec: Vec<u16>, i: u16) -> bool {
        let b = Repr::Arr(to_arr64(&vec));
        check_rank_select!(1, b, i);
        true
    }
}

quickcheck!{
    fn prop_block_rle16_rank0_rank1(vec: Vec<u16>, i: u16) -> TestResult {
        let b = Repr::Run(to_rle16(&vec));
        TestResult::from_bool(b.rank1(i) + b.rank0(i) == i)
    }

    fn prop_block_rle16_rank0_select0(vec: Vec<u16>, i: u16) -> bool {
        let b = Repr::Run(to_rle16(&vec));
        check_rank_select!(0, b, i);
        true
    }

    fn prop_block_rle16_rank1_select1(vec: Vec<u16>, i: u16) -> bool {
        let b = Repr::Run(to_rle16(&vec));
        check_rank_select!(1, b, i);
        true
    }
}

macro_rules! associative {
    ( $x:expr, $y:expr, $z:expr, $fn:ident ) => {
        {
            let r1 = $x.$fn($y).$fn($z).bits();
            let r2 = $x.$fn($y.$fn($z)).bits();
            //r1.count1() == r2.count1() && r1.count0() == r2.count0()
            r1.count() == r2.count()
        }
    }
}

macro_rules! commutative {
    ( $x:expr, $y:expr, $fn:ident ) => {
        {
            let r1 = $x.$fn($y).bits();
            let r2 = $y.$fn($x).bits();
            // r1.count1() == r2.count1() && r1.count0() == r2.count0()
            r1.count() == r2.count()
        }
    }
}

quickcheck!{
    fn prop_set_associativity(vec1: Vec<u32>, vec2: Vec<u32>, vec3: Vec<u32>) -> bool {
        let b1 = &Set::from(vec1);
        let b2 = &Set::from(vec2);
        let b3 = &Set::from(vec3);
        let r1 = associative!(b1, b2, b3, and);
        let r2 = associative!(b1, b2, b3, or);
        let r3 = associative!(b1, b2, b3, xor);
        r1 && r2 && r3
    }
    fn prop_set_commutativity(vec1: Vec<u32>, vec2: Vec<u32>) -> bool {
        let b1 = &Set::from(vec1);
        let b2 = &Set::from(vec2);
        let r1 = commutative!(b1, b2, and);
        let r2 = commutative!(b1, b2, or);
        let r3 = commutative!(b1, b2, xor);
        r1 && r2 && r3
    }
}

#[test]
fn set_bits() {
    let bits = &bits!(1, 3, 1 << 16, 1 << 20, 1 << 30);
    for bit in bits.bits() {
        println!("{}", bit);
    }
    for entry in bits {
        for bit in entry.bits() {
            println!("{}", bit);
        }
    }
}

#[test]
fn set_and() {
    let xs = &bits!(1 << 16, 1 << 20);
    let ys = &bits!(1 << 10, 1 << 11, 1 << 20);
    let and = xs.and(ys);
    let vec = and.bits().collect::<Vec<u32>>();
    assert_eq!(vec.len(), 1);
}

#[test]
fn set_or() {
    let xs = &bits!(1 << 16, 1 << 20);
    let ys = &bits!(1 << 10, 1 << 11, 1 << 20);
    let or = xs.or(ys);
    let vec = or.bits().collect::<Vec<u32>>();
    assert_eq!(vec.len(), 4);
}

#[test]
fn set_and_not() {
    let xs = &bits!(1 << 10, 1 << 11, 1 << 12, 1 << 16, 1 << 20);
    let ys = &bits!(1 << 10, 1 << 11, 1 << 20);
    let and_not = xs.and_not(ys);
    let vec = and_not.bits().collect::<Vec<u32>>();
    assert_eq!(vec.len(), 2);
}

#[test]
fn set_xor() {
    let xs = &bits!(1 << 10, 1 << 11, 1 << 12, 1 << 16, 1 << 20);
    let ys = &bits!(1 << 10, 1 << 11, 1 << 20, 1 << 26, 1 << 30);
    let xor = xs.xor(ys);
    let vec = xor.bits().collect::<Vec<u32>>();
    assert_eq!(vec.len(), 4);
}

fn check_io<T>(w: &T, r: &mut T)
where
    // R: io::Read,
    // W: io::Write,
    T: ReadFrom<io::Cursor<Vec<u8>>> + WriteTo<Vec<u8>>,
{
    let mut buf: Vec<u8> = Vec::with_capacity(2048);
    assert!(w.write_to(&mut buf).is_ok());
    assert!(r.read_from(&mut io::Cursor::new(buf)).is_ok());
}

#[test]
fn block_io() {
    {
        let vec = vec![1u16, 7, 18, 32];
        let len = vec.len();

        let v1 = Seq16::from(vec);
        let mut v2 = Seq16 {
            vector: vec![0; len],
        };
        check_io(&v1, &mut v2);

        assert_eq!(v1, v2);
    }

    {
        let vec = vec![1u16, 6, 18, 1 << 12, 1 << 13];

        let v1 = vec.iter().collect::<Arr64>();
        let mut v2 = Arr64::new();
        check_io(&v1, &mut v2);

        assert_eq!(v1, v2);
    }

    {
        let vec = vec![1u16..=4, 20..=20, 31..=33];

        let v1 = Run16::from(&vec[..]);
        let mut v2 = Run16::new();
        check_io(&v1, &mut v2);

        assert_eq!(v1, v2);
    }
}

quickcheck!{
    fn prop_set_read_write_identity(vec: Vec<u32>) -> bool {
        use std::iter::FromIterator;
        let set1 = Set::from_iter(vec);
        let mut buf = Vec::with_capacity(2048);
        if let Err(_) = set1.write_to(&mut buf) {
            return false
        }
        let mut set2 = Set::new();
        if let Err(_) = set2.read_from(&mut io::Cursor::new(&buf)) {
            return false
        }

        let pop_test = set1.count1() == set2.count1();
        let bit_test = set1.bits().zip(set2.bits()).all(|(a, b)| a == b);
        pop_test && bit_test
    }
}

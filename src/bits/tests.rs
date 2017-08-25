use std::u16;
use quickcheck::TestResult;

use dict::{PopCount, Rank, Select0, Select1};
use bits::{Map32, Map64};
use bits::prim::*;
use bits::pair::*;
use bits::block::*;

fn to_seq16(vec: &Vec<u16>) -> Seq16 {
    Seq16::from(vec.clone())
}
fn to_seq64(vec: &Vec<u16>) -> Seq64 {
    vec.iter().collect()
}
fn to_rle16(vec: &Vec<u16>) -> Rle16 {
    vec.iter().collect()
}

fn all_kind_block(vec: &Vec<u16>) -> (Block, Block, Block) {
    let b1 = Block::Seq16(to_seq16(vec));
    let b2 = Block::Seq64(to_seq64(vec));
    let b3 = Block::Rle16(to_rle16(vec));
    (b1, b2, b3)
}

quickcheck!{
    fn prop_u64_split_merge_identity(w: u64) -> bool {
        w == <u64 as Merge>::merge(w.split())
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
    fn prop_map64_rank0_rank1(vec: Vec<u64>, i: u64) -> TestResult {
        let b = Map64::from(vec);
        TestResult::from_bool(b.rank1(i) + b.rank0(i) == i)
    }

    fn prop_map64_rank0_select0(vec: Vec<u64>, i: u64) -> bool {
        let b = Map64::from(vec);
        check_rank_select!(0, b, i);
        true
    }

    fn prop_map64_rank1_select1(vec: Vec<u64>, i: u64) -> bool {
        let b = Map64::from(vec);
        check_rank_select!(1, b, i);
        true
    }
}

quickcheck!{
    fn prop_map32_rank0_rank1(vec: Vec<u32>, i: u32) -> TestResult {
        let b = Map32::from(vec);
        TestResult::from_bool(b.rank1(i) + b.rank0(i) == i)
    }

    fn prop_map32_rank0_select0(vec: Vec<u32>, i: u32) -> bool {
        let b = Map32::from(vec);
        check_rank_select!(0, b, i);
        true
    }

    fn prop_map32_rank1_select1(vec: Vec<u32>, i: u32) -> bool {
        let b = Map32::from(vec);
        check_rank_select!(1, b, i);
        true
    }
}

quickcheck!{
    fn prop_block_seq16_rank0_rank1(vec: Vec<u16>, i: u16) -> TestResult {
        let b = Block::Seq16(to_seq16(&vec));
        TestResult::from_bool(b.rank1(i) + b.rank0(i) == i)
    }

    fn prop_block_seq16_rank0_select0(vec: Vec<u16>, i: u16) -> bool {
        let b = Block::Seq16(to_seq16(&vec));
        check_rank_select!(0, b, i);
        true
    }

    fn prop_block_seq16_rank1_select1(vec: Vec<u16>, i: u16) -> bool {
        let b = Block::Seq16(to_seq16(&vec));
        check_rank_select!(1, b, i);
        true
    }
}

quickcheck!{
    fn prop_block_seq64_rank0_rank1(vec: Vec<u16>, i: u16) -> TestResult {
        let b = Block::Seq64(to_seq64(&vec));
        TestResult::from_bool(b.rank1(i) + b.rank0(i) == i)
    }
    fn prop_block_seq64_rank0_select0(vec: Vec<u16>, i: u16) -> bool {
        let b = Block::Seq64(to_seq64(&vec));
        check_rank_select!(0, b, i);
        true
    }
    fn prop_block_seq64_rank1_select1(vec: Vec<u16>, i: u16) -> bool {
        let b = Block::Seq64(to_seq64(&vec));
        check_rank_select!(1, b, i);
        true
    }
}

quickcheck!{
    fn prop_block_rle16_rank0_rank1(vec: Vec<u16>, i: u16) -> TestResult {
        let b = Block::Rle16(to_rle16(&vec));
        TestResult::from_bool(b.rank1(i) + b.rank0(i) == i)
    }

    fn prop_block_rle16_rank0_select0(vec: Vec<u16>, i: u16) -> bool {
        let b = Block::Rle16(to_rle16(&vec));
        check_rank_select!(0, b, i);
        true
    }

    fn prop_block_rle16_rank1_select1(vec: Vec<u16>, i: u16) -> bool {
        let b = Block::Rle16(to_rle16(&vec));
        check_rank_select!(1, b, i);
        true
    }
}

macro_rules! associative {
    ( $x:expr, $y:expr, $z:expr, $fn:ident ) => {
        {
            let r1 = $x.$fn($y).$fn($z);
            let r2 = $x.$fn(&$y.$fn($z));
            r1.count1() == r2.count1() && r1.count0() == r2.count0()
        }
    }
}

macro_rules! commutative {
    ( $x:expr, $y:expr, $fn:ident ) => {
        {
            let r1 = $x.$fn(&$y);
            let r2 = $y.$fn(&$x);
            r1.count1() == r2.count1() && r1.count0() == r2.count0()
        }
    }
}

macro_rules! check_block_associativity {
    ( $vec1:expr, $vec2:expr, $vec3:expr, $fn:ident ) => {
        {
            let (a1, a2, a3) = all_kind_block($vec1);
            let (b1, b2, b3) = all_kind_block($vec2);
            let (c1, c2, c3) = all_kind_block($vec3);
            associative!(&a1, &b1, &c1, $fn) &&
            associative!(&a1, &b1, &c2, $fn) &&
            associative!(&a1, &b1, &c3, $fn) &&
            associative!(&a1, &b2, &c1, $fn) &&
            associative!(&a1, &b2, &c2, $fn) &&
            associative!(&a1, &b2, &c3, $fn) &&
            associative!(&a1, &b3, &c1, $fn) &&
            associative!(&a1, &b3, &c2, $fn) &&
            associative!(&a1, &b3, &c3, $fn) &&
            associative!(&a2, &b1, &c1, $fn) &&
            associative!(&a2, &b1, &c2, $fn) &&
            associative!(&a2, &b1, &c3, $fn) &&
            associative!(&a2, &b2, &c1, $fn) &&
            associative!(&a2, &b2, &c2, $fn) &&
            associative!(&a2, &b2, &c3, $fn) &&
            associative!(&a2, &b3, &c1, $fn) &&
            associative!(&a2, &b3, &c2, $fn) &&
            associative!(&a2, &b3, &c3, $fn) &&
            associative!(&a3, &b1, &c1, $fn) &&
            associative!(&a3, &b1, &c2, $fn) &&
            associative!(&a3, &b1, &c3, $fn) &&
            associative!(&a3, &b2, &c1, $fn) &&
            associative!(&a3, &b2, &c2, $fn) &&
            associative!(&a3, &b2, &c3, $fn) &&
            associative!(&a3, &b3, &c1, $fn) &&
            associative!(&a3, &b3, &c2, $fn) &&
            associative!(&a3, &b3, &c3, $fn)
        }
    }
}

macro_rules! check_block_commutativity {
    ( $vec1:expr, $vec2:expr, $fn:ident ) => {
        {
            let (a1, a2, a3) = all_kind_block($vec1);
            let (b1, b2, b3) = all_kind_block($vec2);
            commutative!(&a1, &b1, $fn) &&
            commutative!(&a1, &b2, $fn) &&
            commutative!(&a1, &b3, $fn) &&
            commutative!(&a2, &b1, $fn) &&
            commutative!(&a2, &b2, $fn) &&
            commutative!(&a2, &b3, $fn) &&
            commutative!(&a3, &b1, $fn) &&
            commutative!(&a3, &b2, $fn) &&
            commutative!(&a3, &b3, $fn)
        }
    }
}

quickcheck!{
    fn prop_map64_associativity(vec1: Vec<u64>, vec2: Vec<u64>, vec3: Vec<u64>) -> bool {
        let b1 = &Map64::from(vec1);
        let b2 = &Map64::from(vec2);
        let b3 = &Map64::from(vec3);
        let r1 = associative!(b1, b2, b3, intersection);
        let r2 = associative!(b1, b2, b3, union);
        let r3 = associative!(b1, b2, b3, symmetric_difference);
        r1 && r2 && r3
    }

    fn prop_map64_commutativity(vec1: Vec<u64>, vec2: Vec<u64>) -> bool {
        let b1 = &Map64::from(vec1);
        let b2 = &Map64::from(vec2);
        let r1 = commutative!(b1, b2, intersection);
        let r2 = commutative!(b1, b2, union);
        let r3 = commutative!(b1, b2, symmetric_difference);
        r1 && r2 && r3
    }
}

quickcheck!{
    fn prop_map32_associativity(vec1: Vec<u32>, vec2: Vec<u32>, vec3: Vec<u32>) -> bool {
        let b1 = &Map32::from(vec1);
        let b2 = &Map32::from(vec2);
        let b3 = &Map32::from(vec3);
        let r1 = associative!(b1, b2, b3, intersection);
        let r2 = associative!(b1, b2, b3, union);
        let r3 = associative!(b1, b2, b3, symmetric_difference);
        r1 && r2 && r3
    }

    fn prop_map32_commutativity(vec1: Vec<u32>, vec2: Vec<u32>) -> bool {
        let b1 = &Map32::from(vec1);
        let b2 = &Map32::from(vec2);
        let r1 = commutative!(b1, b2, intersection);
        let r2 = commutative!(b1, b2, union);
        let r3 = commutative!(b1, b2, symmetric_difference);
        r1 && r2 && r3
    }
}

quickcheck!{
    fn prop_block_associativity(vec1: Vec<u16>, vec2: Vec<u16>, vec3: Vec<u16>) -> bool {
        let r1 = check_block_associativity!(&vec1, &vec2, &vec3, intersection);
        let r2 = check_block_associativity!(&vec1, &vec2, &vec3, union);
        let r3 = check_block_associativity!(&vec1, &vec2, &vec3, symmetric_difference);
        r1 && r2 && r3
    }

    fn prop_block_commutativity(vec1: Vec<u16>, vec2: Vec<u16>) -> bool {
        let r1 = check_block_commutativity!(&vec1, &vec2, intersection);
        let r2 = check_block_commutativity!(&vec1, &vec2, union);
        let r3 = check_block_commutativity!(&vec1, &vec2, symmetric_difference);
        r1 && r2 && r3
    }
}

use std::u16;
use quickcheck::TestResult;
use super::*;

fn to_seq16(vec: Vec<u16>) -> Seq16 {
    Seq16::from(vec)
}
fn to_seq64(vec: Vec<u16>) -> Seq64 {
    vec.iter().collect()
}
fn to_rle16(vec: Vec<u16>) -> Rle16 {
    vec.iter().collect()
}

fn all_kind_block(vec: Vec<u16>) -> (Block, Block, Block) {
    let b1 = Block::Seq16(to_seq16(vec.clone()));
    let b2 = Block::Seq64(to_seq64(vec.clone()));
    let b3 = Block::Rle16(to_rle16(vec.clone()));
    (b1, b2, b3)
}

macro_rules! check_rank_select {
    ( 0, $block:expr, $i:expr ) => {
        if let Some(p) = $block.select0($i) {
            if p != 0 {
                return $block.rank0(p) == $i as u32;
            }
        }
    };
    ( 1, $block:expr, $i:expr ) => {
        if let Some(p) = $block.select1($i) {
            if p != 0 {
                return $block.rank1(p) == $i as u32;
            }
        }
    }
}

quickcheck!{
    fn prop_block_all_count_ones(vec: Vec<u16>) -> TestResult {
        let (b1, b2, b3) = all_kind_block(vec);
        TestResult::from_bool(
            b1.count_ones() == b2.count_ones() && b2.count_ones() == b3.count_ones()
        )
    }

    fn prop_block_all_count_zeros(vec: Vec<u16>) -> TestResult {
        let (b1, b2, b3) = all_kind_block(vec);
        TestResult::from_bool(
            b1.count_zeros() == b2.count_zeros() && b2.count_zeros() == b3.count_zeros()
        )
    }

    fn prop_block_all_rank0(vec: Vec<u16>, i: u16) -> TestResult {
        let (b1, b2, b3) = all_kind_block(vec);
        TestResult::from_bool(b1.rank0(i) == b2.rank0(i) && b2.rank0(i) == b3.rank0(i))
    }

    fn prop_block_all_rank1(vec: Vec<u16>, i: u16) -> TestResult {
        let (b1, b2, b3) = all_kind_block(vec);
        TestResult::from_bool(b1.rank1(i) == b2.rank1(i) && b2.rank1(i) == b3.rank1(i))
    }

    fn prop_block_all_select0(vec: Vec<u16>, i: u16) -> TestResult {
        let (b1, b2, b3) = all_kind_block(vec);
        TestResult::from_bool(
            b1.select0(i) == b2.select0(i) && b2.select0(i) == b3.select0(i)
        )
    }

    fn prop_block_all_select1(vec: Vec<u16>, i: u16) -> TestResult {
        let (b1, b2, b3) = all_kind_block(vec);
        TestResult::from_bool(
            b1.select1(i) == b2.select1(i) && b2.select1(i) == b3.select1(i)
        )
    }
}

quickcheck!{
    fn prop_block_seq16_rank0_rank1(vec: Vec<u16>, i: u16) -> TestResult {
        let b = Block::Seq16(to_seq16(vec));
        TestResult::from_bool(b.rank1(i) + b.rank0(i) == i as u32)
    }

    fn prop_block_seq16_rank0_select0(vec: Vec<u16>, i: u16) -> bool {
        let b = Block::Seq16(to_seq16(vec));
        check_rank_select!(0, b, i);
        true
    }

    fn prop_block_seq16_rank1_select1(vec: Vec<u16>, i: u16) -> bool {
        let b = Block::Seq16(to_seq16(vec));
        check_rank_select!(1, b, i);
        true
    }
}

quickcheck!{
    fn prop_block_seq64_rank0_rank1(vec: Vec<u16>, i: u16) -> TestResult {
        let b = Block::Seq64(to_seq64(vec));
        TestResult::from_bool(b.rank1(i) + b.rank0(i) == i as u32)
    }
    fn prop_block_seq64_rank0_select0(vec: Vec<u16>, i: u16) -> bool {
        let b = Block::Seq64(to_seq64(vec));
        check_rank_select!(0, b, i);
        true
    }
    fn prop_block_seq64_rank1_select1(vec: Vec<u16>, i: u16) -> bool {
        let b = Block::Seq64(to_seq64(vec));
        check_rank_select!(1, b, i);
        true
    }
}

quickcheck!{
    fn prop_block_rle16_rank0_rank1(vec: Vec<u16>, i: u16) -> TestResult {
        let b = Block::Rle16(to_rle16(vec));
        TestResult::from_bool(b.rank1(i) + b.rank0(i) == i as u32)
    }

    fn prop_block_rle16_rank0_select0(vec: Vec<u16>, i: u16) -> bool {
        let b = Block::Rle16(to_rle16(vec));
        check_rank_select!(0, b, i);
        true
    }

    fn prop_block_rle16_rank1_select1(vec: Vec<u16>, i: u16) -> bool {
        let b = Block::Rle16(to_rle16(vec));
        check_rank_select!(1, b, i);
        true
    }
}

macro_rules! associative {
    ( $x:expr, $y:expr, $z:expr, $fn:ident ) => {
        {
            let r1 = $x.$fn($y).$fn($z);
            let r2 = $x.$fn(&$y.$fn($z));
            r1.count_ones() == r2.count_ones() && r1.count_zeros() == r2.count_zeros()
        }
    }
}

macro_rules! commutative {
    ( $x:expr, $y:expr, $fn:ident ) => {
        {
            let r1 = $x.$fn(&$y);
            let r2 = $y.$fn(&$x);
            r1.count_ones() == r2.count_ones() && r1.count_zeros() == r2.count_zeros()
        }
    }
}

macro_rules! check_associativity {
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

macro_rules! check_commutativity {
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
    fn prop_block_associativity_intersection(
        vec1: Vec<u16>,
        vec2: Vec<u16>,
        vec3: Vec<u16>
    ) -> bool {
        check_associativity!(vec1, vec2, vec3, intersection)
    }

    fn prop_block_commutativity_intersection(
        vec1: Vec<u16>,
        vec2: Vec<u16>
    ) -> bool {
        check_commutativity!(vec1, vec2, intersection)
    }

    fn prop_block_associativity_union(
        vec1: Vec<u16>,
        vec2: Vec<u16>,
        vec3: Vec<u16>
    ) -> bool {
        check_associativity!(vec1, vec2, vec3, union)
    }

    fn prop_block_commutativity_union(
        vec1: Vec<u16>,
        vec2: Vec<u16>
    ) -> bool {
        check_commutativity!(vec1, vec2, union)
    }

    fn prop_block_associativity_symmetric_difference(
        vec1: Vec<u16>,
        vec2: Vec<u16>,
        vec3: Vec<u16>
    ) -> bool {
        check_associativity!(vec1, vec2, vec3, symmetric_difference)
    }

    fn prop_block_commutativity_symmetric_difference(
        vec1: Vec<u16>,
        vec2: Vec<u16>
    ) -> bool {
        check_commutativity!(vec1, vec2, symmetric_difference)
    }
}

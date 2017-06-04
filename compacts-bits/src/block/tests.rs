#![allow(non_snake_case, dead_code)]

extern crate rand;

use self::rand::Rng;

use super::*;

use pairwise::*;
use rank::*;
use select::*;

#[test]
fn block_intersection() {
    bitops_test!(VEC & VEC);
    bitops_test!(VEC & MAP);
    bitops_test!(MAP & VEC);
    bitops_test!(MAP & MAP);
    bitops_test!(MIN_VEC & MIN_VEC);
    bitops_test!(MIN_VEC & MIN_MAP);
    bitops_test!(MIN_MAP & MIN_VEC);
    bitops_test!(MIN_MAP & MIN_MAP);
    bitops_test!(MAX_VEC & MAX_VEC);
    bitops_test!(MAX_VEC & MAX_MAP);
    bitops_test!(MAX_MAP & MAX_VEC);
    bitops_test!(MAX_MAP & MAX_MAP);
}

#[test]
fn block_union() {
    bitops_test!(VEC | VEC);
    bitops_test!(VEC | MAP);
    bitops_test!(MAP | VEC);
    bitops_test!(MAP | MAP);
    bitops_test!(MIN_VEC | MIN_VEC);
    bitops_test!(MIN_VEC | MIN_MAP);
    bitops_test!(MIN_MAP | MIN_VEC);
    bitops_test!(MIN_MAP | MIN_MAP);
    bitops_test!(MAX_VEC | MAX_VEC);
    bitops_test!(MAX_VEC | MAX_MAP);
    bitops_test!(MAX_MAP | MAX_VEC);
    bitops_test!(MAX_MAP | MAX_MAP);
}

#[test]
fn block_symmetric_difference() {
    bitops_test!(VEC ^ VEC);
    bitops_test!(VEC ^ MAP);
    bitops_test!(MAP ^ VEC);
    bitops_test!(MAP ^ MAP);
    bitops_test!(MIN_VEC ^ MIN_VEC);
    bitops_test!(MIN_VEC ^ MIN_MAP);
    bitops_test!(MIN_MAP ^ MIN_VEC);
    bitops_test!(MIN_MAP ^ MIN_MAP);
    bitops_test!(MAX_VEC ^ MAX_VEC);
    bitops_test!(MAX_VEC ^ MAX_MAP);
    bitops_test!(MAX_MAP ^ MAX_VEC);
    bitops_test!(MAX_MAP ^ MAX_MAP);
}

#[test]
fn block_difference() {
    bitops_test!(VEC - VEC);
    bitops_test!(VEC - MAP);
    bitops_test!(MAP - VEC);
    bitops_test!(MAP - MAP);
    bitops_test!(MIN_VEC - MIN_VEC);
    bitops_test!(MIN_VEC - MIN_MAP);
    bitops_test!(MIN_MAP - MIN_VEC);
    bitops_test!(MIN_MAP - MIN_MAP);
    bitops_test!(MAX_VEC - MAX_VEC);
    bitops_test!(MAX_VEC - MAX_MAP);
    bitops_test!(MAX_MAP - MAX_VEC);
    bitops_test!(MAX_MAP - MAX_MAP);
}


#[derive(Debug)]
struct RankSelect {
    size: usize,
    block: Block,
}

impl RankSelect {
    fn run<R: Rng>(size: usize, rng: &mut R) {
        let t = Self::new(size, rng);
        t.max_rank_is_equals_to_ones();
        t.rank_select_identity(rng);
    }

    fn new<R: Rng>(size: usize, rng: &mut R) -> Self {
        let mut block = Block::new();
        for _ in 0..size {
            //block.insert(rng.gen_range(0, ::std::u16::MAX));
            block.insert(rng.gen_range(0, (size - 1) as u16));
        }

        println!("({:>5?}) before {:?} {:?}", size, block, block.mem());
        block.optimize();
        println!("({:>5?}) after  {:?} {:?}", size, block, block.mem());

        RankSelect { size, block }
    }

    fn max_rank_is_equals_to_ones(&self) {
        let ones = self.block.count_ones();
        let rank = self.block.rank1(!0u16);
        assert_eq!(ones, rank, "{:?}", self);
    }
    fn rank_select_identity<R: Rng>(&self, rng: &mut R) {
        {
            let c = if self.block.count_ones() == 0 {
                0
            } else {
                rng.gen_range(0, self.block.count_ones())
            };
            let s1 = self.block.select1(c as u16).unwrap_or(0);
            let r1 = self.block.rank1(s1);

            if r1 != 0 {
                assert_eq!(c, r1 - 1, "{:?}", self);
            } else {
                assert_eq!(c, 0, "{:?}", self);
            }
        }

        {
            let c = if self.block.count_zeros() == 0 {
                0
            } else {
                rng.gen_range(0, self.block.count_zeros())
            };

            let s0 = self.block.select0(c as u16).unwrap_or(0);
            let r0 = self.block.rank0(s0);

            if r0 != 0 {
                assert_eq!(c, r0 - 1, "{:?}", self.block);
            } else {
                assert_eq!(c, 0, "{:?}", self.block);
            }
        }
    }
}

#[test]
fn random_rank_select() {
    use self::inner::*;
    let mut rng = rand::thread_rng();
    let lenghs = vec![0,
                      Seq16::THRESHOLD as u64,
                      Seq16::THRESHOLD as u64 * 2,
                      Block::CAPACITY as u64 / 2,
                      Block::CAPACITY as u64,
                      rng.gen_range(10, Seq16::THRESHOLD as u64),
                      rng.gen_range(Seq16::THRESHOLD as u64 + 1, Block::CAPACITY as u64 - 1)];
    for &size in lenghs.iter() {
        RankSelect::run(size as usize, &mut rng);
    }
}

#[test]
fn block_insert_remove() {
    let mut b = Block::new();
    let mut i = 0u16;
    while (i as usize) < inner::Seq16::THRESHOLD {
        assert!(b.insert(i), format!("insert({:?}) failed", i));
        assert!(b.contains(i));
        i += 1;
    }
    assert_eq!(i as usize, inner::Seq16::THRESHOLD);
    assert_eq!(b.count_ones(), inner::Seq16::THRESHOLD as u32);

    while (i as u32) < Block::CAPACITY {
        assert!(b.insert(i), "insert failed");
        assert!(b.contains(i), "insert ok, but not contains");
        if i == !0 {
            break;
        }
        i += 1;
    }

    println!("{:?}", b);
    assert!(b.count_ones() == Block::CAPACITY);
    b.optimize();
    assert!(b.count_ones() == Block::CAPACITY);
    println!("{:?}", b);

    while i > 0 {
        assert!(b.remove(i), format!("remove({:?}) failed", i));
        assert!(!b.contains(i));
        i -= 1;
    }
    assert!(b.remove(i), format!("remove({:?}) failed", i));
    assert_eq!(i, 0);

    assert!(b.count_ones() == 0);
    b.optimize();
    assert!(b.count_ones() == 0);
}

#[test]
fn block_clone() {
    let b1 = inner::Seq64::new();
    let mut b2 = b1.clone();
    b2.insert(0);
    b2.insert(1);
    assert!(b1.weight == 0, "{:?} {:?}", b1.weight, b2.weight);
    assert!(b2.weight == 2, "{:?} {:?}", b1.weight, b2.weight);
}

macro_rules! test_rank {
    ( $block:ident, $repr:ident ) => {
        {
            use std::u16;
            let vec = vec![0...1, 4...5, 8...9, (u16::MAX - 100)...u16::MAX];
            let rle16 = inner::Rle16::from(&vec[..]);
            let block = Block::$block(inner::$repr::from(rle16));
            assert_eq!(1, block.rank1(0));
            assert_eq!(0, block.rank0(0));
            assert_eq!(2, block.rank1(1));
            assert_eq!(0, block.rank0(1));
            assert_eq!(2, block.rank1(2));
            assert_eq!(1, block.rank0(2));
            assert_eq!(2, block.rank1(3));
            assert_eq!(2, block.rank0(3));
            assert_eq!(3, block.rank1(4));
            assert_eq!(2, block.rank0(4));
            assert_eq!(4, block.rank1(5));
            assert_eq!(2, block.rank0(5));
            assert_eq!(4, block.rank1(6));
            assert_eq!(3, block.rank0(6));
            assert_eq!(4, block.rank1(7));
            assert_eq!(4, block.rank0(7));
            assert_eq!(5, block.rank1(8));
            assert_eq!(4, block.rank0(8));
            assert_eq!(6, block.rank1(9));
            assert_eq!(4, block.rank0(9));
            assert_eq!(6, block.rank1(10));
            assert_eq!(5, block.rank0(10));
            assert_eq!(6, block.rank1(100));
            assert_eq!(95, block.rank0(100));
            assert_eq!(6, block.rank1(200));
            assert_eq!(195, block.rank0(200));
            assert_eq!(block.count_ones(), block.rank1(65535));
            assert_eq!(block.count_zeros(), block.rank0(65535));
        }
    }
}

macro_rules! test_select {
    ( $block:ident, $repr:ident ) => {
        {
            use std::u16;
            let vec = vec![0...1, 4...5, 8...9, (u16::MAX - 100)...u16::MAX];
            let rle16 = inner::Rle16::from(&vec[..]);
            let block = Block::$block(inner::$repr::from(rle16));
            assert_eq!(Some(0), block.select1(0));
            assert_eq!(Some(2), block.select0(0));
            assert_eq!(Some(1), block.select1(1));
            assert_eq!(Some(3), block.select0(1));
            assert_eq!(Some(4), block.select1(2));
            assert_eq!(Some(6), block.select0(2));
            assert_eq!(Some(5), block.select1(3));
            assert_eq!(Some(7), block.select0(3));
            assert_eq!(Some(8), block.select1(4));
            assert_eq!(Some(10), block.select0(4));
            assert_eq!(Some(9), block.select1(5));
            assert_eq!(Some(11), block.select0(5));
            assert_eq!(Some(u16::MAX - 100), block.select1(6));
            assert_eq!(Some(12), block.select0(6));
        }
    }
}

#[test]
fn block_rank_select() {
    test_rank!(Vec16, Seq16);
    test_rank!(Vec64, Seq64);
    test_rank!(Rle16, Rle16);

    test_select!(Vec16, Seq16);
    test_select!(Vec64, Seq64);
    test_select!(Rle16, Rle16);
}

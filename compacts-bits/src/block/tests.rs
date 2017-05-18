#![allow(non_snake_case, dead_code)]

extern crate rand;
use self::rand::Rng;

use super::*;
use dict::*;
use pairwise::PairwiseWith;

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
        let mut block = Block::with_capacity(size);
        for _ in 0..size {
            block.insert(rng.gen());
        }
        RankSelect { size, block }
    }

    fn max_rank_is_equals_to_ones(&self) {
        let ones = self.block.count1();
        let rank = self.block.rank1(!0u16);
        assert_eq!(ones, rank, "{:?}", self);
    }
    fn rank_select_identity<R: Rng>(&self, rng: &mut R) {
        {
            let c = if self.block.count1() == 0 {
                0
            } else {
                rng.gen_range(0, self.block.count1())
            };
            let s1 = self.block.select1(c as u16).unwrap_or(0);
            let r1 = self.block.rank1(s1);
            assert_eq!(c, r1, "{:?}", self);
        }
        {
            let c = if self.block.count0() == 0 {
                0
            } else {
                rng.gen_range(0, self.block.count0())
            };
            let s0 = self.block.select0(c as u16).unwrap_or(0);
            let r0 = self.block.rank0(s0);
            assert_eq!(c, r0, "{:?}", self);
        }
    }
}

// #[test]
// fn block_rank_select() {
//     let mut rng = rand::thread_rng();
//     let lenghs = vec![0,
//                       THRESHOLD as u64,
//                       THRESHOLD as u64 * 2,
//                       Block::CAPACITY as u64 / 2,
//                       Block::CAPACITY as u64,
//                       rng.gen_range(10, THRESHOLD as u64),
//                       rng.gen_range(THRESHOLD as u64 + 1, Block::CAPACITY as u64 - 1)];

//     for &size in lenghs.iter() {
//         RankSelect::run(size as usize, &mut rng);
//     }
// }

#[test]
fn block_insert_remove() {
    let mut b = Block::new();
    let mut i = 0u16;
    while (i as usize) < THRESHOLD {
        assert!(b.insert(i), format!("insert({:?}) failed", i));
        assert!(b.contains(i));
        i += 1;
    }
    assert!(b.is_sorted());
    assert_eq!(i as usize, THRESHOLD);
    assert_eq!(b.count1(), THRESHOLD as u32);

    while (i as u32) < Block::CAPACITY {
        assert!(b.insert(i), "insert failed");
        assert!(b.contains(i), "insert ok, but not contains");
        if i == !0 {
            break;
        }
        i += 1;
    }

    assert!(b.is_sorted() && b.count1() == Block::CAPACITY);
    b.optimize();
    assert!(b.is_mapped() && b.count1() == Block::CAPACITY);

    while i > 0 {
        assert!(b.remove(i), format!("remove({:?}) failed", i));
        assert!(!b.contains(i));
        i -= 1;
    }
    assert!(b.remove(i), format!("remove({:?}) failed", i));
    assert_eq!(i, 0);

    assert!(b.is_mapped() && b.count1() == 0);
    b.optimize();
    assert!(b.is_sorted() && b.count1() == 0);
}

#[test]
fn block_clone() {
    let b1 = Bucket::<u64>::new();
    let mut b2 = b1.clone();
    b2.insert(0);
    b2.insert(1);
    assert!(b1.weight == 0, "{:?} {:?}", b1.weight, b2.weight);
    assert!(b2.weight == 2, "{:?} {:?}", b1.weight, b2.weight);
}

extern crate cwt;

use self::cwt::{Bits, SplitMerge, Rank0, Rank1, Select1};

struct RankSelect<T> {
    bvec: T,
    case: (usize, Option<u64>),
}

impl<T> RankSelect<T>
    where T: Bits + Rank0 + Rank1 + Select1
{
    fn run(&self) {
        let &RankSelect { ref bvec, ref case } = self;
        let &(arg, want) = case;

        let s9 = bvec.select1(arg);
        assert_eq!(s9, want);

        let r9 = bvec.rank1(<T as Bits>::SIZE as usize);
        assert_eq!(r9, bvec.ones());

        if let Some(s9) = s9 {
            assert_eq!(bvec.rank1(s9 as usize), arg as u64);
        }
    }
}

#[cfg_attr(rustfmt, rustfmt_skip)]
static TESTS_64: &[RankSelect<u64>] =
    &[RankSelect{bvec: 0b_0000000000_0000000000, case: (0, None)},
      RankSelect{bvec: 0b_0000100101_1000111001, case: (1, Some(3))},
      RankSelect{bvec: 0b_0000100101_1000111001, case: (2, Some(4))},
      RankSelect{bvec: 0b_0000100101_1000111001, case: (3, Some(5))},
      RankSelect{bvec: 0b_0000100101_1000111001, case: (4, Some(9))},
      RankSelect{bvec: 0b_0000100101_1000111001, case: (5, Some(10))},
      RankSelect{bvec: 0b_0000100101_1000111001, case: (6, Some(12))},
      RankSelect{bvec: 0b_0000100101_1000111001, case: (7, Some(15))},
      RankSelect{bvec: 0b_0000100101_0000000000, case: (0, Some(10))},
      RankSelect{bvec: 0b_0000100101_0000000000, case: (1, Some(12))},
      RankSelect{bvec: 0b_0000100101_0000000000, case: (2, Some(15))},
      RankSelect{bvec: 0b_0000000000_0000000001, case: (0, Some(0))},
      RankSelect{bvec: 0b_0000100101_0000000000, case: (3, None)},
      RankSelect{bvec: 0b_0000000000_0000000001, case: (1, None)}];


#[test]
fn rank_select() {
    for test in TESTS_64 {
        test.run();
    }
}

#[test]
fn split_merge() {
    let w = 0b_1100_u64;
    assert!(w == <u64 as SplitMerge<u32>>::merge(w.split()));
}

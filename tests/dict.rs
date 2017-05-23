extern crate compacts;
use compacts::dict::*;
use compacts::dict::prim::*;

struct RankSelect {
    word: u64,
    case: (u32, Option<u32>),
}

impl RankSelect {
    fn run(&self) {
        let RankSelect { word, case } = *self;
        let (arg, want) = case;
        {
            let r9 = word.rank1(64u32);
            let ranked = &word as &Ranked<u32, Weight = u32>;
            assert_eq!(r9, ranked.count1());
            assert_eq!(r9, ranked.rank1(ranked.size()));
            assert_eq!(ranked.size(), ranked.count1() + ranked.count0());

            assert_eq!(word.rank1(64u32), Bits(word).rank(TRUE, 64u32));
            assert_eq!(word.rank0(64u32), Bits(word).rank(FALSE, 64u32));
        }
        {
            let s9 = word.select1(arg);
            assert_eq!(s9, want);
            assert_eq!(word.select1(arg), Bits(word).select(TRUE, arg));
            assert_eq!(word.select0(arg), Bits(word).select(FALSE, arg));
        }

        if let Some(s9) = word.select1(arg) {
            assert_eq!(word.rank1(s9), arg + 1);
            assert_eq!(Bits(word).rank(TRUE, s9), arg + 1);
        }
    }
}

#[cfg_attr(rustfmt, rustfmt_skip)]
static TESTS_64: &[RankSelect] =
    &[RankSelect{word: 0b_0000000000_0000000000, case: (0, None)},
      RankSelect{word: 0b_0000100101_1000111001, case: (0, Some(0))},
      RankSelect{word: 0b_0000100101_1000111001, case: (1, Some(3))},
      RankSelect{word: 0b_0000100101_1000111001, case: (2, Some(4))},
      RankSelect{word: 0b_0000100101_1000111001, case: (3, Some(5))},
      RankSelect{word: 0b_0000100101_1000111001, case: (4, Some(9))},
      RankSelect{word: 0b_0000100101_1000111001, case: (5, Some(10))},
      RankSelect{word: 0b_0000100101_1000111001, case: (6, Some(12))},
      RankSelect{word: 0b_0000100101_1000111001, case: (7, Some(15))},
      RankSelect{word: 0b_0000100101_0000000000, case: (0, Some(10))},
      RankSelect{word: 0b_0000100101_0000000000, case: (1, Some(12))},
      RankSelect{word: 0b_0000100101_0000000000, case: (2, Some(15))},
      RankSelect{word: 0b_1000100101_0000000000, case: (3, Some(19))},
      RankSelect{word: 0b_0000100101_0000000000, case: (3, None)},
      RankSelect{word: 0b_0000000000_0000000001, case: (1, None)}];


#[test]
fn word_rank_select() {
    for test in TESTS_64 {
        test.run();
    }
}

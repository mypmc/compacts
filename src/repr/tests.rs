#![allow(non_snake_case, dead_code)]

extern crate rand;
use self::rand::Rng;

extern crate test;
use self::test::Bencher;

use super::*;

fn generate_repr<R: Rng>(size: usize, rng: &mut R) -> Repr {
    let mut repr = Repr::with_capacity(size);
    for _ in 0..size {
        repr.insert(rng.gen());
    }
    repr
}

#[derive(Debug)]
struct RankSelect {
    size: usize,
    repr: Repr,
}

impl RankSelect {
    fn run<R: Rng>(size: usize, rng: &mut R) {
        let t = Self::new(size, rng);
        t.max_rank_is_equals_to_ones();
        t.rank_select_identity(rng);
    }

    fn new<R: Rng>(size: usize, rng: &mut R) -> RankSelect {
        let repr = generate_repr(size, rng);
        RankSelect { size, repr }
    }
    fn max_rank_is_equals_to_ones(&self) {
        let ones = self.repr.ones();
        let rank = self.repr.rank1(Repr::SIZE);
        assert_eq!(ones, rank, "{:?}", self);
    }
    fn rank_select_identity<R: Rng>(&self, rng: &mut R) {
        let c = if self.repr.ones() == 0 {
            0
        } else {
            rng.gen_range(0, self.repr.ones())
        };
        let s = self.repr.select1(c).unwrap_or(0);
        let r = self.repr.rank1(s);
        assert_eq!(c, r, "{:?}", self);
    }
}

#[cfg_attr(rustfmt, rustfmt_skip)]
static LENGTHS: &'static [usize] = &[0, Repr::VEC_SIZE, Repr::VEC_SIZE * 2, Repr::SIZE / 2, Repr::SIZE];

#[test]
fn repr_rank_select() {
    let mut rng = rand::thread_rng();
    let lens = {
        let mut vec = vec![rng.gen_range(10, Repr::VEC_SIZE),
                           rng.gen_range(Repr::VEC_SIZE + 1, Repr::SIZE - 1)];
        vec.extend_from_slice(LENGTHS);
        vec.sort();
        vec
    };
    for &size in lens.iter() {
        RankSelect::run(size, &mut rng);
    }
}

struct IterTest<'a> {
    bits: &'a [u64],
    ones: usize,
    dirs: &'a [Option<u16>],
}
impl<'a> IterTest<'a> {
    fn run(bits: &'a [u64], dirs: &'a [Option<u16>]) {
        Self::new(bits, dirs).test()
    }
    fn new(bits: &'a [u64], dirs: &'a [Option<u16>]) -> IterTest<'a> {
        let ones = bits.iter().fold(0, |acc, &x| acc + x.ones());
        IterTest { bits, ones, dirs }
    }
    fn test(&mut self) {
        let mut iter = Iter::map(self.bits, self.ones);
        for (i, &want) in self.dirs.iter().enumerate() {
            let got = iter.next();
            assert_eq!(got, want, "{:?}", i);
        }
    }
}

#[test]
fn repr_iter_next() {
    {
        let bits = &[1 | 1 << 63, 1 | 1 << 63, 1 | 1 << 63];
        let dirs = &[Some(0), Some(63), Some(64), Some(127), Some(128), Some(191), None, None];
        IterTest::run(bits, dirs);
    }
}

struct TestOp<'a> {
    lhs: &'a Repr,
    rhs: &'a Repr,
    ops: &'a Fn(&Repr, &Repr) -> Repr,
}

impl<'a> TestOp<'a> {
    fn run(&self) -> Repr {
        let &TestOp { lhs, rhs, ops } = self;
        ops(lhs, rhs)
    }
}

macro_rules! init_repr {
    ( VEC; $repr: ident, $rng: expr ) => {
        let size = $rng.gen_range(0, Repr::VEC_SIZE);
        init_repr!($repr, size, $rng);
    };
    ( MAP; $repr: ident, $rng: expr ) => {
        let size = $rng.gen_range(Repr::VEC_SIZE, Repr::SIZE);
        init_repr!($repr, size, $rng);
    };
    ( $repr: ident, $size: expr, $rng: expr ) => {
        let $repr = &generate_repr( $size, &mut $rng );
    };
}
macro_rules! init_bitops {
    ( $test: ident, $lhs: ident & $rhs: ident ) => {
        init_bitops!( $test, $lhs, $rhs, &(|x, y| x & y) );
    };
    ( $test: ident, $lhs: ident | $rhs: ident ) => {
        init_bitops!( $test, $lhs, $rhs, &(|x, y| x | y) );
    };
    ( $test: ident, $lhs: ident ^ $rhs: ident ) => {
        init_bitops!( $test, $lhs, $rhs, &(|x, y| x ^ y) );
    };

    ( $test: ident, $lhs: ident, $rhs: ident, $ops: expr ) => {
        let $test = TestOp { lhs: $lhs, rhs: $rhs, ops: $ops };
    };
}

macro_rules! bitops {
    ( $this: ident & $that: ident; $lhs: ident, $rhs: ident, $test: ident ) => {
        let mut rng = rand::thread_rng();
        init_repr!($this; $lhs, rng);
        init_repr!($that; $rhs, rng);
        init_bitops!($test, $lhs & $rhs);
    };
    ( $this: ident | $that: ident; $lhs: ident, $rhs: ident, $test: ident ) => {
        let mut rng = rand::thread_rng();
        init_repr!($this; $lhs, rng);
        init_repr!($that; $rhs, rng);
        init_bitops!($test, $lhs | $rhs);
    };
    ( $this: ident ^ $that: ident; $lhs: ident, $rhs: ident, $test: ident ) => {
        let mut rng = rand::thread_rng();
        init_repr!($this; $lhs, rng);
        init_repr!($that; $rhs, rng);
        init_bitops!($test, $lhs ^ $rhs);
    };
}

macro_rules! bitops_test {
    ( $this: ident & $that: ident ) => {
        bitops!($this & $that; lhs, rhs, test);
        let bitand = test.run();
        for bit in &bitand {
            assert!(lhs.contains(bit) && rhs.contains(bit),
                    "bitand={bitand:?} lhs={lhs:?} rhs={rhs:?}",
                    bitand=bitand, lhs=lhs, rhs=rhs);
        }
        let pair = pair!(intersection, lhs, rhs);
        for (i, j) in pair.zip(&bitand){
            assert!(i == j, "i:{:?} j:{:?}", i, j);
            assert!((lhs.contains(i) && rhs.contains(i)) && (lhs.contains(j) && rhs.contains(j)),
                    "bitand={bitand:?} lhs={lhs:?} rhs={rhs:?}",
                    bitand=bitand, lhs=lhs, rhs=rhs);
        }
    };
    ( $this: ident | $that: ident ) => {
        bitops!($this | $that; lhs, rhs, test);
        let bitor = test.run();
        for bit in &bitor {
            assert!(lhs.contains(bit) || rhs.contains(bit),
                    "bitor={bitor:?} lhs={lhs:?} rhs={rhs:?}",
                    bitor=bitor, lhs=lhs, rhs=rhs);
        }
        let pair = pair!(union, lhs, rhs);
        for (i, j) in pair.zip(&bitor){
            assert!(i == j, "i:{:?} j:{:?}", i, j);
            assert!((lhs.contains(i) || rhs.contains(i)) && (lhs.contains(j) || rhs.contains(j)),
                    "bitor={bitor:?} lhs={lhs:?} rhs={rhs:?}",
                    bitor=bitor, lhs=lhs, rhs=rhs);
        }
    };
    ( $this: ident ^ $that: ident ) => {
        bitops!($this ^ $that; lhs, rhs, test);
        let bitxor = test.run();
        for bit in &bitxor {
            assert!(!(lhs.contains(bit) && rhs.contains(bit)),
                    "bitxor={bitxor:?} lhs={lhs:?} rhs={rhs:?}",
                    bitxor=bitxor, lhs=lhs, rhs=rhs);
        }
        let pair = pair!(symmetric_difference, lhs, rhs);
        for (i, j) in pair.zip(&bitxor){
            assert!(i == j, "i:{:?} j:{:?}", i, j);
            assert!(!(lhs.contains(i) && rhs.contains(i)) && !(lhs.contains(j) && rhs.contains(j)),
                    "bitxor={bitxor:?} lhs={lhs:?} rhs={rhs:?}",
                    bitxor=bitxor, lhs=lhs, rhs=rhs);
        }
    };
}

#[test]
fn repr_bitop_AND() {
    bitops_test!(VEC & VEC);
    bitops_test!(VEC & MAP);
    bitops_test!(MAP & VEC);
    bitops_test!(MAP & MAP);
}

#[test]
fn repr_bitop_OR() {
    bitops_test!(VEC | VEC);
    bitops_test!(VEC | MAP);
    bitops_test!(MAP | VEC);
    bitops_test!(MAP | MAP);
}

#[test]
fn repr_bitop_XOR() {
    bitops_test!(VEC ^ VEC);
    bitops_test!(VEC ^ MAP);
    bitops_test!(MAP ^ VEC);
    bitops_test!(MAP ^ MAP);
}

#[test]
fn repr_insert_remove() {
    // let _ = env_logger::init();

    let mut b = Repr::none();
    let mut i = 0u16;
    while i < Repr::VEC_SIZE as u16 {
        assert!(b.insert(i), format!("insert({:?}) failed", i));
        assert!(b.contains(i));
        i += 1;
    }
    assert_eq!(b.ones(), Repr::VEC_SIZE);
    while i < Repr::SIZE as u16 {
        assert!(b.insert(i), "insert failed");
        assert!(b.contains(i), "insert ok, but not contains");
        i += 1;
    }
    assert!(b.insert(i), format!("insert({:?}) failed", i));
    assert!(b.contains(i));

    b.optimize();

    assert_eq!(i as usize, Repr::VEC_SIZE);
    assert_eq!(b.ones(), Repr::VEC_SIZE + 1);

    while i > 0 {
        assert!(b.remove(i), format!("remove({:?}) failed", i));
        assert!(!b.contains(i));
        i -= 1;
    }
    assert!(b.remove(i), format!("remove({:?}) failed", i));
    assert_eq!(i, 0);
    assert_eq!(b.ones(), 0);

    b.optimize();
    assert_eq!(0, b.ones());
}

macro_rules! run_bench_bitops {
    ( $this: ident & $that: ident; $bench: expr ) => {
        bitops!($this & $that; lhs, rhs, test); $bench.iter(|| test.run());
    };
    ( $this: ident | $that: ident; $bench: expr ) => {
        bitops!($this | $that; lhs, rhs, test); $bench.iter(|| test.run());
    };
    ( $this: ident ^ $that: ident; $bench: expr ) => {
        bitops!($this | $that; lhs, rhs, test); $bench.iter(|| test.run());
    };
}

#[bench]
fn VEC_and_VEC(bench: &mut Bencher) {
    run_bench_bitops!(VEC & VEC; bench);
}
#[bench]
fn VEC_and_MAP(bench: &mut Bencher) {
    run_bench_bitops!(VEC & MAP; bench);
}
#[bench]
fn MAP_and_VEC(bench: &mut Bencher) {
    run_bench_bitops!(MAP & VEC; bench);
}
#[bench]
fn MAP_and_MAP(bench: &mut Bencher) {
    run_bench_bitops!(MAP & MAP; bench);
}

#[bench]
fn VEC_or_VEC(bench: &mut Bencher) {
    run_bench_bitops!(VEC | VEC; bench);
}
#[bench]
fn VEC_or_MAP(bench: &mut Bencher) {
    run_bench_bitops!(VEC | MAP; bench);
}
#[bench]
fn MAP_or_VEC(bench: &mut Bencher) {
    run_bench_bitops!(MAP | VEC; bench);
}
#[bench]
fn MAP_or_MAP(bench: &mut Bencher) {
    run_bench_bitops!(MAP | MAP; bench);
}

#[bench]
fn VEC_xor_VEC(bench: &mut Bencher) {
    run_bench_bitops!(VEC ^ VEC; bench);
}
#[bench]
fn VEC_xor_MAP(bench: &mut Bencher) {
    run_bench_bitops!(VEC ^ MAP; bench);
}
#[bench]
fn MAP_xor_VEC(bench: &mut Bencher) {
    run_bench_bitops!(MAP ^ VEC; bench);
}
#[bench]
fn MAP_xor_MAP(bench: &mut Bencher) {
    run_bench_bitops!(MAP ^ MAP; bench);
}

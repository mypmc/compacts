#![allow(non_snake_case, dead_code)]

extern crate rand;
use self::rand::Rng;

extern crate test;
use self::test::Bencher;

use {bits, Bucket, BucketIter};
use {Bounded, PopCount, Rank1, Select1};
use bucket::pair;
use super::VEC_CAPACITY;

fn generate_bucket<R: Rng>(size: usize, rng: &mut R) -> Bucket {
    let mut bucket = Bucket::with_capacity(size);
    for _ in 0..size {
        bucket.insert(rng.gen());
    }
    bucket
}

#[derive(Debug)]
struct RankSelect {
    size: usize,
    bucket: Bucket,
}

impl RankSelect {
    fn run<R: Rng>(size: usize, rng: &mut R) {
        let t = Self::new(size, rng);
        t.max_rank_is_equals_to_ones();
        t.rank_select_identity(rng);
    }

    fn new<R: Rng>(size: usize, rng: &mut R) -> RankSelect {
        let bucket = generate_bucket(size, rng);
        RankSelect { size, bucket }
    }
    fn max_rank_is_equals_to_ones(&self) {
        let ones = self.bucket.ones();
        let rank = self.bucket.rank1(Bucket::CAPACITY as usize);
        assert_eq!(ones, rank, "{:?}", self);
    }
    fn rank_select_identity<R: Rng>(&self, rng: &mut R) {
        let c = if self.bucket.ones() == 0 {
            0
        } else {
            rng.gen_range(0, self.bucket.ones())
        };
        let s = self.bucket.select1(c as usize).unwrap_or(0);
        let r = self.bucket.rank1(s as usize);
        assert_eq!(c, r, "{:?}", self);
    }
}

#[cfg_attr(rustfmt, rustfmt_skip)]
static LENGTHS: &'static [u64] =
    &[0, VEC_CAPACITY as u64, (VEC_CAPACITY * 2) as u64, Bucket::CAPACITY / 2, Bucket::CAPACITY];

#[test]
fn bucket_rank_select() {
    let mut rng = rand::thread_rng();
    let lens = {
        let mut vec = vec![rng.gen_range(10, VEC_CAPACITY as u64),
                           rng.gen_range(VEC_CAPACITY as u64 + 1, Bucket::CAPACITY - 1)];
        vec.extend_from_slice(LENGTHS);
        vec.sort();
        vec
    };
    for &size in lens.iter() {
        RankSelect::run(size as usize, &mut rng);
    }
}

struct IterTest<'a> {
    bits: &'a [u64],
    ones: u64,
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
        let pop = bits::Count::<u16>::new(self.ones);
        let mut iter = BucketIter::map(self.bits, &pop);
        for (i, &want) in self.dirs.iter().enumerate() {
            let got = iter.next();
            assert_eq!(got, want, "{:?}", i);
        }
    }
}

#[test]
fn bucket_iter_next() {
    {
        let bits = &[1 | 1 << 63, 1 | 1 << 63, 1 | 1 << 63];
        let dirs = &[Some(0), Some(63), Some(64), Some(127), Some(128), Some(191), None, None];
        IterTest::run(bits, dirs);
    }
}

struct TestOp<'a> {
    lhs: &'a Bucket,
    rhs: &'a Bucket,
    ops: &'a Fn(&Bucket, &Bucket) -> Bucket,
}

impl<'a> TestOp<'a> {
    fn run(&self) -> Bucket {
        let &TestOp { lhs, rhs, ops } = self;
        ops(lhs, rhs)
    }
}

macro_rules! init_bucket {
    ( MIN_VEC; $bucket: ident, $rng: expr ) => {
        let size = 0;
        init_bucket!($bucket, size as usize, $rng);
    };
    ( MAX_VEC; $bucket: ident, $rng: expr ) => {
        let size = VEC_CAPACITY;
        init_bucket!($bucket, size as usize, $rng);
    };
    ( MIN_MAP; $bucket: ident, $rng: expr ) => {
        let size = VEC_CAPACITY + 1;
        init_bucket!($bucket, size as usize, $rng);
    };
    ( MAX_MAP; $bucket: ident, $rng: expr ) => {
        let size = Bucket::CAPACITY - 1;
        init_bucket!($bucket, size as usize, $rng);
    };
    ( VEC; $bucket: ident, $rng: expr ) => {
        let size = $rng.gen_range(0, VEC_CAPACITY);
        init_bucket!($bucket, size as usize, $rng);
    };
    ( MAP; $bucket: ident, $rng: expr ) => {
        let size = $rng.gen_range(VEC_CAPACITY as u64, Bucket::CAPACITY);
        init_bucket!($bucket, size as usize, $rng);
    };
    ( $bucket: ident, $size: expr, $rng: expr ) => {
        let $bucket = &generate_bucket( $size, &mut $rng );
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
        init_bucket!($this; $lhs, rng);
        init_bucket!($that; $rhs, rng);
        init_bitops!($test, $lhs & $rhs);
    };
    ( $this: ident | $that: ident; $lhs: ident, $rhs: ident, $test: ident ) => {
        let mut rng = rand::thread_rng();
        init_bucket!($this; $lhs, rng);
        init_bucket!($that; $rhs, rng);
        init_bitops!($test, $lhs | $rhs);
    };
    ( $this: ident ^ $that: ident; $lhs: ident, $rhs: ident, $test: ident ) => {
        let mut rng = rand::thread_rng();
        init_bucket!($this; $lhs, rng);
        init_bucket!($that; $rhs, rng);
        init_bitops!($test, $lhs ^ $rhs);
    };
}

macro_rules! bitops_test {
    ( $this: ident & $that: ident ) => {
        bitops!($this & $that; lhs, rhs, test);
        let bitand = test.run();
        for bit in &bitand {
            assert!(lhs.contains(bit) && rhs.contains(bit),
                    "AND ({bit:?}): result={result:?} lhs={lhs:?} rhs={rhs:?}",
                    bit=bit, result=bitand, lhs=lhs, rhs=rhs);
        }
        let pair = pair!(intersection, lhs, rhs);
        let c = pair.collect::<Bucket>().ones();
        assert!(c == bitand.ones(),
                "{c:?} {ones:?} lhs={lhs:?} rhs={rhs:?}",
                c = c, ones = bitand.ones(), lhs=lhs, rhs=rhs);
    };
    ( $this: ident | $that: ident ) => {
        bitops!($this | $that; lhs, rhs, test);
        let bitor = test.run();
        for bit in &bitor {
            assert!(lhs.contains(bit) || rhs.contains(bit),
                    "OR ({bit:?}): result={result:?} lhs={lhs:?} rhs={rhs:?}",
                    bit=bit, result=bitor, lhs=lhs.contains(bit), rhs=rhs.contains(bit));
        }
        let pair = pair!(union, lhs, rhs);
        let c = pair.collect::<Bucket>().ones();
        assert!(c == bitor.ones(), "{c:?} {ones:?} lhs={lhs:?} rhs={rhs:?}",
                c=c,ones=bitor.ones(), lhs=lhs, rhs=rhs);
    };
    ( $this: ident ^ $that: ident ) => {
        bitops!($this ^ $that; lhs, rhs, test);
        let bitxor = test.run();
        for bit in &bitxor {
            assert!(!(lhs.contains(bit) && rhs.contains(bit)),
                    "XOR ({bit:?}): result={result:?} lhs={lhs:?} rhs={rhs:?}",
                    bit=bit, result=bitxor, lhs=lhs.contains(bit), rhs=rhs.contains(bit));
        }
        let pair = pair!(symmetric_difference, lhs, rhs);
        let c = pair.collect::<Bucket>().ones();
        assert!(c == bitxor.ones(), "{c:?} {ones:?} lhs={lhs:?} rhs={rhs:?}",
                c=c,ones=bitxor.ones(), lhs=lhs, rhs=rhs);
    };
}

#[test]
fn bucket_bitop_AND() {
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
fn bucket_bitop_OR() {
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
fn bucket_bitop_XOR() {
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
fn bucket_insert_remove() {
    let mut b = Bucket::new();
    let mut i = 0u16;
    while (i as usize) < VEC_CAPACITY {
        assert!(b.insert(i), format!("insert({:?}) failed", i));
        assert!(b.contains(i));
        i += 1;
    }
    assert_eq!(i as usize, VEC_CAPACITY);
    assert_eq!(b.ones(), VEC_CAPACITY as u64);

    while (i as u64) < Bucket::CAPACITY {
        assert!(b.insert(i), "insert failed");
        assert!(b.contains(i), "insert ok, but not contains");
        if i == <u16 as Bounded>::MAX {
            break;
        }
        i += 1;
    }

    b.optimize();
    assert_eq!(b.ones(), Bucket::CAPACITY);

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

#[test]
fn pop_count_max() {
    {
        let cnt: u64 = 1 << 16;
        let pop = bits::Count::<u16>::new(cnt);
        assert!(pop.value() == cnt);
    }
    {
        let cnt: u64 = 1 << 32;
        let pop = bits::Count::<u32>::new(cnt);
        assert!(pop.value() == cnt);
    }
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

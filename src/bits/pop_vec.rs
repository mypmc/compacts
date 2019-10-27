use std::{
    cmp::Ordering,
    fmt::{self, Debug},
    iter::{once, repeat, repeat_with},
    ops::{Add, AddAssign, RangeBounds, Sub, SubAssign},
};

use Ordering::{Equal as EQ, Greater as GT, Less as LT};

use crate::{bits, fenwick::FenwickTree, num, num::Int, ops::*};

const UPPER_BLOCK: usize = 1 << 32;
const SUPER_BLOCK: usize = 2048;
const BASIC_BLOCK: usize = 512;

const SUPERS: usize = UPPER_BLOCK / SUPER_BLOCK; // 2097152

// const BASICS: usize = SUPER_BLOCK / BASIC_BLOCK; // 4

/// `BitVec<T>` is `Vec<T>` with the index for `rank1` and `rank0`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pop<T> {
    samples: Samples,
    bits: Vec<T>,
}

// L0: cumulative     absolute counts
// L1: cumulative     relative counts
// L2: non-cumulative relative counts
#[derive(Debug, Clone, PartialEq, Eq)]
struct Samples {
    uppers: FenwickTree<u64>,
    // L1 and L2 are interleaved into one vector,
    // each L1 entries is followed by its L2 index entries.
    lowers: Vec<FenwickTree<L1L2>>,
}

// /// `BitArray<T>` is a freezed `BitVec` with the extra index for `select1`.
// #[derive(Debug, Clone, PartialEq, Eq)]
// pub struct PopArray<T> {
//     samples: CombinedSamples,
//     data: Vec<T>,
// }

// #[derive(Debug, Clone, PartialEq, Eq)]
// struct CombinedSamples {
//     pops: Samples,
//     pos1: Vec<Vec<u32>>,
// }

/// Interleaves L1[i] and L2[i] into 64bit word.
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
struct L1L2(u64);

/// (upper_blocks, lower_blocks, super_blocks)
#[inline]
fn sampling_blocks(bits: usize) -> (usize, usize, usize) {
    let upper_blocks = bits::blocks_by(bits, UPPER_BLOCK);
    let super_blocks = bits::blocks_by(bits, SUPER_BLOCK);
    let (lower_blocks, remain) = divrem!(super_blocks, SUPERS);
    assert_eq!(upper_blocks, lower_blocks + (remain > 0) as usize);
    (upper_blocks, lower_blocks, remain)
}

impl<T: FixedBits> Pop<T> {
    /// Returns an empty `BitVec`.
    ///
    /// ```
    /// # use compacts::ops::Bits;
    /// # type BitVec<T> = compacts::Pop<T>;
    /// let bv = BitVec::<u64>::new(1000);
    /// assert!(bv.len() >= 1000);
    /// ```
    pub fn new(len: usize) -> Self {
        Pop {
            samples: Samples::none(len),
            bits: bits::sized(len),
        }
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.bits.capacity() * T::SIZE
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.bits.len() * T::SIZE
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Resizes the `BitVec` in-place so that `BitVec` has at least `new_len` bits.
    #[inline]
    pub fn resize(&mut self, min: usize) {
        let cur = self.bits.size();
        self.samples.resize(cur, min);

        let len = bits::blocks_by(min, T::SIZE);
        self.bits.resize_with(len, T::none);
    }

    /// Swaps bit at `i` by `bit` and returns the previous value.
    fn swap(&mut self, i: usize, bit: bool) -> bool {
        BOUNDS_CHECK!(i < self.bits.size());
        let (i, o) = divrem!(i, T::SIZE);
        let cur = self.bits[i].bit(o);
        if !cur && bit {
            self.bits[i].put1(o);
        } else if cur && !bit {
            self.bits[i].put0(o);
        }
        cur
    }
}

impl<T: FixedBits> Bits for Pop<T> {
    #[inline]
    fn size(&self) -> usize {
        self.bits.size()
    }

    #[inline]
    fn bit(&self, i: usize) -> bool {
        self.bits.bit(i)
    }

    #[inline]
    fn count1(&self) -> usize {
        debug_assert_eq!(self.samples.count1(), self.bits.count1());
        self.samples.count1()
    }

    #[inline]
    fn rank1<R: RangeBounds<usize>>(&self, range: R) -> usize {
        let rank = |p0| -> usize {
            if p0 == self.bits.size() {
                self.samples.count1()
            } else {
                let (q0, r0) = divrem!(p0, UPPER_BLOCK);
                let (q1, r1) = divrem!(r0, SUPER_BLOCK);
                let (q2, r2) = divrem!(r1, BASIC_BLOCK);

                let up = &self.samples.uppers;
                let lo = &self.samples.lowers[q0];

                let c0: u64 = up.sum(q0);
                let c1: u64 = lo.sum_by(q1, |l1l2| l1l2.l1());
                let c2 = lo.tree[q1 + 1].l2(q2);

                num::cast::<u64, usize>(c0 + c1 + c2) + self.bits.rank1(p0 - r2..p0)
            }
        };

        match bits::to_exclusive(&range, self.bits.size()).expect("out of bounds") {
            (0, k) => rank(k),
            (i, j) => rank(j) - rank(i),
        }
    }

    #[inline]
    fn select1(&self, n: usize) -> Option<usize> {
        let uppers = &self.samples.uppers;
        let lowers = &self.samples.lowers;
        let mut remain = num::cast::<usize, u64>(n);

        uppers.search(remain + 1).ok().map(|l0| {
            remain -= uppers.sum::<u64>(l0);

            let l1 = lowers[l0].search_by(remain + 1, |l1l2| l1l2.l1()).unwrap();
            remain -= lowers[l0].sum_by::<u64, _, _>(l1, |l1l2| l1l2.l1());

            let l2 = {
                let l1l2 = lowers[l0].tree[l1 + 1]; // 0 is sentinel
                let mut index = 0;

                for &count in &[l1l2.l2_0(), l1l2.l2_1(), l1l2.l2_2()] {
                    if remain < count {
                        break;
                    }
                    remain -= count;
                    index += 1;
                }
                index
            };

            let mut offset = l0 * UPPER_BLOCK + l1 * SUPER_BLOCK + l2 * BASIC_BLOCK;

            // maybe we can optimize this loop
            loop {
                let dst = std::cmp::min(self.bits.size(), offset + <u64 as Int>::BITS);
                let len = dst - offset;
                let sum = num::cast(self.bits.rank1(offset..dst));

                if remain < sum {
                    let w = self.bits.getn::<u64>(offset, len);
                    break offset + w.select1(remain as usize).unwrap();
                }

                remain -= sum;
                offset += len;
            }
        })
    }
}

impl<T: FixedBits> BitsMut for Pop<T> {
    #[inline]
    fn put1(&mut self, p0: usize) {
        if !self.swap(p0, true) {
            self.samples.add(p0, 1);
        }
    }

    #[inline]
    fn put0(&mut self, p0: usize) {
        if self.swap(p0, false) {
            self.samples.sub(p0, 1);
        }
    }
}

impl Samples {
    fn none(len: usize) -> Self {
        let (up, lo, sb) = sampling_blocks(len);

        Samples {
            uppers: FenwickTree::with_default(up),
            lowers: {
                let base = repeat_with(|| FenwickTree::with_default(SUPERS)).take(lo);
                if sb == 0 {
                    base.collect()
                } else {
                    base.chain(once(FenwickTree::with_default(sb))).collect()
                }
            },
        }
    }

    fn resize(&mut self, bit_len: usize, new_len: usize) {
        let (uppers, lowers, supers) = sampling_blocks(new_len);

        let up = &mut self.uppers;
        let lo = &mut self.lowers;

        match bit_len.cmp(&new_len) {
            EQ => { /* do nothing */ }

            LT if up.len() < uppers => {
                let diff = uppers - up.len();
                up.extend(repeat(0).take(diff));

                // None if bit_len is 0
                if let Some(lo_last) = lo.last_mut() {
                    lo_last.extend_by_default(SUPERS - lo_last.len(), L1L2::l1);
                }

                lo.extend(repeat_with(|| FenwickTree::with_default(SUPERS)).take(diff - 1));

                if supers > 0 {
                    lo.push(FenwickTree::with_default(supers));
                } else {
                    lo.push(FenwickTree::with_default(SUPERS));
                }
            }

            LT if up.len() == uppers => {
                let last = lo.last_mut().unwrap(); // lo must not be empty
                let diff = if supers > 0 {
                    supers - last.len()
                } else {
                    SUPERS - last.len()
                };
                last.extend_by_default(diff, L1L2::l1);
            }

            GT => {
                up.tree.truncate(uppers + 1);
                lo.truncate(lowers + (supers > 0) as usize);

                if supers > 0 {
                    lo[lowers].tree.truncate(supers + 1);
                }
            }

            _ => unreachable!(),
        }

        if cfg!(test) {
            // dbg!(self.up.len(), self.lo.len(), uppers, lowers, supers);
            assert_eq!(self.uppers.len(), self.lowers.len());
        }
    }

    fn add(&mut self, p0: usize, delta: u64) {
        self.uppers.tree[0] += delta; // To use sentinel effectively

        let (q0, r0) = divrem!(p0, UPPER_BLOCK);
        let (q1, r1) = divrem!(r0, SUPER_BLOCK);
        let q2 = r1 / BASIC_BLOCK;

        self.uppers.add(q0, delta);
        {
            let q1 = &mut self.lowers[q0].tree[q1 + 1]; // skip sentinel
            let mut arr = q1.split();
            if q2 + 1 < arr.len() {
                arr[q2 + 1] += delta;
            }
            *q1 = L1L2::merge(arr);
        }
        self.lowers[q0].add(q1, delta);
    }

    fn sub(&mut self, p0: usize, delta: u64) {
        self.uppers.tree[0] -= delta;

        let (q0, r0) = divrem!(p0, UPPER_BLOCK);
        let (q1, r1) = divrem!(r0, SUPER_BLOCK);
        let q2 = r1 / BASIC_BLOCK;

        self.uppers.sub(q0, delta);
        {
            let q1 = &mut self.lowers[q0].tree[q1 + 1];
            let mut arr = q1.split();
            if q2 + 1 < arr.len() {
                arr[q2 + 1] -= delta;
            }
            *q1 = L1L2::merge(arr);
        }
        self.lowers[q0].sub(q1, delta);
    }

    #[inline]
    fn count1(&self) -> usize {
        num::cast(self.uppers.tree[0])
    }
}

impl Default for L1L2 {
    fn default() -> L1L2 {
        L1L2(0)
    }
}

impl Debug for L1L2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("L1L2")
            .field("l1", &self.l1())
            .field("l2_0", &self.l2_0())
            .field("l2_1", &self.l2_1())
            .field("l2_2", &self.l2_2())
            .finish()
    }
}

impl Add<u64> for L1L2 {
    type Output = L1L2;
    fn add(self, delta: u64) -> Self::Output {
        let L1L2(l1l2) = self;

        // L1 should fit in u32
        debug_assert_eq!(l1l2 + delta, {
            let l1 = self.l1() + delta;
            l1l2 & 0b_00111111111111111111111111111111_00000000000000000000000000000000_u64 | l1
        });

        L1L2(l1l2 + delta)
    }
}

impl AddAssign<u64> for L1L2 {
    fn add_assign(&mut self, delta: u64) {
        self.0 += delta;
    }
}

impl Sub<u64> for L1L2 {
    type Output = L1L2;
    fn sub(self, delta: u64) -> Self::Output {
        let L1L2(l1l2) = self;

        // L1 should fit in u32
        debug_assert_eq!(l1l2 - delta, {
            let l1 = self.l1() - delta;
            l1l2 & 0b_00111111111111111111111111111111_00000000000000000000000000000000_u64 | l1
        });

        L1L2(l1l2 - delta)
    }
}

impl SubAssign<u64> for L1L2 {
    fn sub_assign(&mut self, delta: u64) {
        self.0 -= delta;
    }
}

impl L1L2 {
    #[inline]
    fn merge(mut arr: [u64; 4]) -> Self {
        assert!(arr[0] < UPPER_BLOCK as u64);
        assert!(arr[1] < 1024 && arr[2] < 1024 && arr[3] < 1024);
        arr[0] |= arr[1] << 32;
        arr[0] |= arr[2] << 42;
        arr[0] |= arr[3] << 52;
        L1L2(arr[0])
    }

    #[inline]
    fn split(self) -> [u64; 4] {
        [self.l1(), self.l2_0(), self.l2_1(), self.l2_2()]
    }

    #[inline]
    fn l1(self) -> u64 {
        let L1L2(l1l2) = self;
        (l1l2 & 0b_00000000000000000000000000000000_11111111111111111111111111111111_u64)
    }

    #[inline]
    fn l2_0(self) -> u64 {
        let L1L2(l1l2) = self;
        (l1l2 & 0b_00000000000000000000001111111111_00000000000000000000000000000000_u64) >> 32
    }

    #[inline]
    fn l2_1(self) -> u64 {
        let L1L2(l1l2) = self;
        (l1l2 & 0b_00000000000011111111110000000000_00000000000000000000000000000000_u64) >> 42
    }

    #[inline]
    fn l2_2(self) -> u64 {
        let L1L2(l1l2) = self;
        (l1l2 & 0b_00111111111100000000000000000000_00000000000000000000000000000000_u64) >> 52
    }

    fn l2(self, index: usize) -> u64 {
        match index {
            0 => 0,
            1 => self.l2_0(),
            2 => self.l2_0() + self.l2_1(),
            3 => self.l2_0() + self.l2_1() + self.l2_2(),
            _ => unreachable!("basic block: index out of bounds"),
        }
    }
}

#[cfg(test)]
mod samples {
    use super::*;
    // use rand::prelude::*;

    #[test]
    fn pop_none() {
        let pop = Pop::<u64>::new(0);
        dbg!(&pop.samples.uppers.len(), pop.samples.lowers.len());
        let pop = Pop::<u64>::new(2048);
        dbg!(&pop.samples.uppers.len(), pop.samples.lowers.len());
        let pop = Pop::<u64>::new(1 << 32);
        dbg!(&pop.samples.uppers.len(), pop.samples.lowers.len());
    }

    #[test]
    fn pop_addsub() {
        let mut pop = Pop::<u64>::new(65536 + 1024);
        for i in 0..512 {
            pop.put1(i);
        }
        assert_eq!(512, pop.count1());

        for i in (1024..1536).step_by(2) {
            pop.put1(i);
        }

        assert_eq!(512 + 256, pop.count1());

        assert_eq!(pop.rank1(..512), 512);
        assert_eq!(pop.rank1(1024..1536), 256);
    }

    #[test]
    fn pop_select() {
        let mut pop = Pop::<u64>::new(65536 + 1024);
        for i in (0..65536).step_by(64) {
            pop.put1(i);
        }

        // dbg!(pop.select1(0));
        // dbg!(pop.select1(1));
        // dbg!(pop.select1(2));
        // dbg!(pop.select1(3));
        // dbg!(pop.select1(4));

        dbg!(pop.bits.select1(990), pop.select1(990));
        dbg!(pop.bits.select1(991), pop.select1(991));
        dbg!(pop.bits.select1(992), pop.select1(992));
        dbg!(pop.bits.select1(993), pop.select1(993));
        dbg!(pop.bits.select1(994), pop.select1(994));

        // for i in 0..65536 / 64 {
        //     dbg!(i);
        //     assert_eq!(Some(i * 64), pop.select1(i));
        // }
    }

    #[test]
    fn pop_resize() {
        let mut pop_vec = Pop::<u64>::new(0);

        let mut resize = |size| {
            pop_vec.resize(size);
            assert_eq!(pop_vec.rank0(..size), size);
        };

        let vec = vec![
            0,
            0,
            1 << 30,
            0,
            (1 << 33) + 1000,
            1 << 31,
            10000,
            (1 << 32) - 2048,
            1 << 32,
            1 << 22,
            1 << 33,
            10,
            1 << 32,
            1 << 31,
            (1 << 32) - 2048,
            (1 << 23) + 1024,
            1 << 33,
            (1 << 23) + 2048,
            1 << 31,
            1 << 30,
            1 << 30,
        ];

        for &n in &vec {
            resize(n);
        }
    }

    // #[test]
    // fn suc_vec3() {
    //     let size = 65536 * 3;
    //     let mut bit_vec = Pop::<u64>::none(size);

    //     bit_vec.put1(4096 - 9);
    //     bit_vec.put1(4096 - 5);
    //     bit_vec.put1(4096);

    //     dbg!(bit_vec.rank1(..4096));
    //     dbg!(bit_vec.rank1(..));

    //     // bit_vec.rank1_at(4096);
    //     // bit_vec.rank1_at(4100);
    // }

    // #[test]
    // fn suc_vec2() {
    //     let size = 65536 * 10;
    //     let mut bit_vec = Pop::<u64>::none(size);

    //     for i in (0..size).step_by(127) {
    //         bit_vec.put1(i);
    //     }

    //     // let seq = (0..size).step_by(512).collect::<Vec<_>>();
    //     // dbg!(&seq);

    //     // [0, 1024, 2048, 3072, 4096, 5120, 6144, ..
    //     // bit_vec.rank1_at(2048);
    //     for i in (0..size).step_by(100) {
    //         dbg!(bit_vec.rank1(..i));
    //     }
    // }
}

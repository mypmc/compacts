#![allow(missing_docs)]

use std::ops::RangeBounds;

use crate::{
    bits::Words,
    num::{self, cast, Int, Word},
    ops::*,
};

/// An immutable and uncompressed bit sequence.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BitArray<T> {
    // the number of enabled bits
    ones: u64,
    data: Vec<T>,

    // samping values for rank
    sum_samples: SumSamples,
    // samping values for select
    idx_samples: IdxSamples,
}

/// A sampling values of `rank1`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct SumSamples {
    // L0: cumulative     absolute counts
    // L1: cumulative     relative counts
    // L2: non-cumulative relative counts
    // L1 and L2 are interleaved into one vector,
    // each L1 entries is followed by its L2 index entries.
    l0s: Vec<u64>,
    l1l2s: Vec<L1L2>,
}

/// An interleaved value of L1[i] and L2[i] of `RankSamples`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
struct L1L2(u64);

/// A sampling values of `select1`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct IdxSamples {
    idxs: Vec<Vec<u32>>,
}

const UPPER_BLOCK: usize = 1 << 32;
const SUPER_BLOCK: usize = 2048;
const BASIC_BLOCK: usize = 512;

const SAMPLE_SIZE: usize = 8192;

const NUM_SB: usize = (1 << 32) / 2048; // 2097152
const NUM_BB: usize = 2048 / 512;

fn words<T: Word>(slice: &[T], chunk_bits: usize) -> impl Iterator<Item = Option<&[T]>> {
    assert!(chunk_bits % T::BITS == 0 && chunk_bits <= 65536);
    slice.chunks(chunk_bits / T::BITS).map(Some)
}

impl<T: Word> From<Vec<T>> for BitArray<T> {
    fn from(data: Vec<T>) -> Self {
        let (ones, sum_samples, idx_samples) = {
            let slice = data.as_slice();
            samples(slice.size(), words(slice, SUPER_BLOCK))
        };

        debug_assert_eq!(ones, data.count1() as u64);
        BitArray {
            ones,
            data,
            sum_samples,
            idx_samples,
        }
    }
}

impl<T: Words> From<Vec<Option<Box<T>>>> for BitArray<Option<Box<T>>> {
    fn from(data: Vec<Option<Box<T>>>) -> Self {
        let (ones, sum_samples, idx_samples) = {
            let slice = data.as_slice();
            samples(slice.size(), {
                use std::iter::repeat;
                type FixedBits<'a, W> = Option<&'a [W]>;
                assert!(T::BITS % SUPER_BLOCK == 0 && SUPER_BLOCK <= 65536);

                slice.iter().flat_map(move |entry| {
                    if let Some(b) = entry.as_ref() {
                        Box::new(words(b.as_ref_words(), SUPER_BLOCK))
                            as Box<dyn Iterator<Item = FixedBits<'_, T::Word>> + '_>
                    } else {
                        Box::new(repeat(None).take(T::BITS / SUPER_BLOCK))
                            as Box<dyn Iterator<Item = FixedBits<'_, T::Word>> + '_>
                    }
                })
            })
        };

        debug_assert_eq!(ones, data.count1() as u64);
        BitArray {
            ones,
            data,
            sum_samples,
            idx_samples,
        }
    }
}

fn samples<'a, T, I>(size: usize, supers: I) -> (u64, SumSamples, IdxSamples)
where
    T: Word,
    I: Iterator<Item = Option<&'a [T]>>,
{
    use crate::bits::blocks_by;
    let mut l0s = vec![0; blocks_by(size, UPPER_BLOCK)];
    let mut l1l2s = vec![L1L2(0); blocks_by(size, SUPER_BLOCK)];

    let mut idxs = vec![Vec::new(); l0s.len()];
    let mut ones = 0i64; // max is 1<<63

    const ISIZE: i64 = SAMPLE_SIZE as i64;
    let mut cur = 0;
    let mut pre = 0;

    for (i, chunk) in supers.enumerate() {
        let basics = {
            let mut bbs = [0; NUM_BB];
            if let Some(slice) = chunk.as_ref() {
                for (i, bb) in slice.chunks(BASIC_BLOCK / T::SIZE).enumerate() {
                    bbs[i] = bb.count1() as u64;
                }
            }
            bbs
        };

        let pop_count = {
            if i % NUM_SB == 0 {
                l0s[i / NUM_SB] = cur;
                pre = cur;
            }
            let l1 = cur - pre;

            l1l2s[i] = L1L2::interleave(l1, basics[0], basics[1], basics[2]);

            let sum = basics.iter().sum::<u64>();
            cur += sum;
            sum as usize
        };

        {
            let sample_index = i / NUM_SB;
            let select_index = ((-ones) % ISIZE + ISIZE) % ISIZE; // modulo

            if (select_index as usize) < pop_count {
                let chunk = chunk.expect("pop_count > 0");
                let offset = i * SUPER_BLOCK;
                let select = chunk.select1(select_index as usize).unwrap();
                idxs[sample_index].push(cast(offset + select - sample_index * UPPER_BLOCK));
            }

            if i % NUM_SB == NUM_SB - 1 {
                ones = 0;
            } else {
                ones += pop_count as i64;
            }
        }
    }

    (cur, SumSamples { l0s, l1l2s }, IdxSamples { idxs })
}

#[allow(clippy::large_digit_groups)]
impl L1L2 {
    fn interleave(mut l1: u64, l2_0: u64, l2_1: u64, l2_2: u64) -> Self {
        assert!(l1 < UPPER_BLOCK as u64 && l2_0 < 1024 && l2_1 < 1024 && l2_2 < 1024);
        l1 |= l2_0 << 32;
        l1 |= l2_1 << 42;
        l1 |= l2_2 << 52;
        L1L2(l1)
    }

    #[inline(always)]
    fn l1(self) -> u64 {
        (self.0 & 0b_00000000000000000000000000000000_11111111111111111111111111111111_u64)
    }

    #[inline(always)]
    fn l2_0(self) -> u64 {
        (self.0 & 0b_00000000000000000000001111111111_00000000000000000000000000000000_u64) >> 32
    }
    #[inline(always)]
    fn l2_1(self) -> u64 {
        (self.0 & 0b_00000000000011111111110000000000_00000000000000000000000000000000_u64) >> 42
    }
    #[inline(always)]
    fn l2_2(self) -> u64 {
        (self.0 & 0b_00111111111100000000000000000000_00000000000000000000000000000000_u64) >> 52
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

impl<T: FixedBits> Bits for BitArray<T> {
    #[inline(always)]
    fn size(&self) -> usize {
        self.data.as_slice().size()
    }

    #[inline(always)]
    fn bit(&self, i: usize) -> bool {
        self.data.bit(i)
    }

    #[inline(always)]
    fn count1(&self) -> usize {
        cast(self.ones)
    }

    #[inline]
    fn rank1<R: RangeBounds<usize>>(&self, range: R) -> usize {
        let rank = |p0| {
            if p0 == self.size() {
                self.count1()
            } else {
                let hi = &self.sum_samples.l0s[p0 / UPPER_BLOCK];
                let (q1, r1) = divrem!(p0, SUPER_BLOCK);
                let (q2, r2) = divrem!(r1, BASIC_BLOCK);
                let lo: &L1L2 = &self.sum_samples.l1l2s[q1];
                cast::<u64, usize>(hi + lo.l1() + lo.l2(q2)) + self.data.rank1(p0 - r2..p0)
            }
        };

        match super::to_exclusive(&range, self.size()).expect("out of bounds") {
            (0, k) => rank(k),
            (i, j) => rank(j) - rank(i),
        }
    }

    fn select1(&self, nth: usize) -> Option<usize> {
        if nth >= self.count1() {
            return None;
        };

        let mut remain = cast::<usize, u64>(nth);

        let l0s = &self.sum_samples.l0s;
        let l1l2s = &self.sum_samples.l1l2s;
        let idxs = &self.idx_samples.idxs;

        // Lookup in L0 to find the right UpperBlock
        let l0_index = num::binary_search(0, l0s.len(), |k| remain < l0s[k]) - 1;
        remain -= l0s[l0_index];

        let (l1l2_index, l2_index) = {
            // Lookup in sampling answers to find the nearby LowerBlock
            let (n, m) = {
                let l1_samples = &idxs[l0_index];
                let skipped = l0_index * UPPER_BLOCK;
                let i = cast::<u64, usize>(remain / SAMPLE_SIZE as u64);
                let j = i + 1;
                let min = l1_samples.get(i).map_or(0, |&k| cast::<u32, usize>(k));
                let max = l1_samples
                    .get(j)
                    .map_or(UPPER_BLOCK, |&k| cast::<u32, usize>(k));
                assert!(min < max);
                (
                    (skipped + min) / SUPER_BLOCK,
                    (skipped + max) / SUPER_BLOCK + 1,
                )
            };

            // Lookup in L1 to find the right LowerBlock
            let i = std::cmp::min(n, l1l2s.len() - 1);
            let j = std::cmp::min(m, l1l2s.len());
            let l1l2_index = num::binary_search(i, j, |k| remain < l1l2s[k].l1()) - 1;

            let l1l2 = l1l2s[l1l2_index];
            let l1 = l1l2.l1();
            let l2 = [l1l2.l2_0(), l1l2.l2_1(), l1l2.l2_2()];

            assert!(remain >= l1);
            remain -= l1;

            // Lookup in L2 to find the right BasicBlock
            let l2_index = {
                let mut index = 0;
                for &l2 in l2.iter() {
                    if remain < l2 {
                        break;
                    }
                    remain -= l2;
                    index += 1;
                }
                index
            };

            (l1l2_index, l2_index)
        };

        let mut pos = {
            let sb = l1l2_index * SUPER_BLOCK;
            let bb = l2_index * BASIC_BLOCK;
            sb + bb
        };

        assert!(remain <= 512);

        let step = u64::BITS;
        let bits = self.data.as_slice();
        loop {
            let dst = std::cmp::min(bits.size(), pos + step);
            let len = dst - pos;
            let sum = cast(bits.rank1(pos..dst));
            if remain < sum {
                let w = bits.getn::<u64>(pos, len);
                pos += w.select1(remain as usize).unwrap();
                break;
            }
            remain -= sum;
            pos += len;
        }

        Some(pos)
    }

    #[inline(always)]
    fn getn<W: Word>(&self, i: usize, n: usize) -> W {
        self.data.getn(i, n)
    }
}

use std::{ops, u16};
use std::iter::FromIterator;
use super::{ArrBlock, RunBlock, SeqBlock};

impl<'r> From<&'r ArrBlock> for SeqBlock {
    fn from(arr: &ArrBlock) -> Self {
        let mut seq = SeqBlock::with_capacity(arr.weight as usize);
        let iter = arr.bitmap.iter();
        for (i, &w) in iter.enumerate().filter(|&(_, v)| *v != 0) {
            for p in 0..64 {
                if w & (1 << p) != 0 {
                    let bit = i * 64 + p;
                    debug_assert!(bit <= u16::MAX as usize);
                    seq.insert(bit as u16);
                }
            }
        }
        seq
    }
}

impl<'r> From<&'r RunBlock> for SeqBlock {
    fn from(run: &'r RunBlock) -> Self {
        let mut seq = SeqBlock::with_capacity(run.weight as usize);
        for range in &run.ranges {
            seq.vector.extend(range.clone());
        }
        seq
    }
}

#[cfg(test)]
impl From<Vec<u16>> for SeqBlock {
    fn from(mut vector: Vec<u16>) -> Self {
        vector.sort();
        vector.dedup();
        assert!(vector.len() <= super::SEQ_MAX_LEN);
        SeqBlock { vector }
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(needless_range_loop))]
fn insert_range(bitmap: &mut [u64], range: &super::Range) {
    let s = range.start as usize;
    let e = range.end as usize;

    let (head, last) = {
        let h = !0 << (s % 64);
        let l = !0 >> ((-((e + 1) as i64)) as u64 % 64);
        (h, l)
    };

    let sq = s / 64;
    let eq = e / 64;

    if sq == eq {
        bitmap[sq] |= head & last;
    } else {
        bitmap[sq] |= head;
        bitmap[eq] |= last;
        for i in (sq + 1)..eq {
            bitmap[i] = !0;
        }
    }
}

impl<'r> From<&'r SeqBlock> for ArrBlock {
    fn from(seq: &'r SeqBlock) -> Self {
        seq.vector.iter().collect()
    }
}

impl<'r> From<&'r RunBlock> for ArrBlock {
    fn from(run: &'r RunBlock) -> Self {
        let mut arr = ArrBlock::new();
        arr.weight = run.weight;
        for r in &run.ranges {
            insert_range(&mut *arr.bitmap, r);
        }
        arr
    }
}

impl<'a> FromIterator<&'a u16> for ArrBlock {
    fn from_iter<I>(i: I) -> Self
    where
        I: IntoIterator<Item = &'a u16>,
    {
        let mut arr = ArrBlock::new();
        for &item in i {
            arr.insert(item);
        }
        arr
    }
}
impl FromIterator<u16> for ArrBlock {
    fn from_iter<I>(i: I) -> Self
    where
        I: IntoIterator<Item = u16>,
    {
        let mut arr = ArrBlock::new();
        for item in i {
            arr.insert(item);
        }
        arr
    }
}

impl<'a> From<&'a SeqBlock> for RunBlock {
    fn from(vec16: &'a SeqBlock) -> Self {
        vec16.vector.iter().collect()
    }
}

impl<'a> From<&'a ArrBlock> for RunBlock {
    fn from(arr: &'a ArrBlock) -> Self {
        let mut run = RunBlock::new();
        for (i, &bit) in arr.bitmap.iter().enumerate().filter(|&(_, &v)| v != 0) {
            for p in 0..64 {
                if bit & (1 << p) != 0 {
                    let x = (i as u16 * 64) + p;
                    run.insert(x);
                }
            }
        }
        run
    }
}

impl<'a> From<&'a [super::Range]> for RunBlock {
    fn from(slice: &'a [super::Range]) -> Self {
        let mut rle16 = RunBlock {
            weight: 0,
            ranges: Vec::with_capacity(slice.len()),
        };
        for r in slice {
            let w = u32::from(r.end - r.start) + 1;
            rle16.weight += w;
            rle16.ranges.push(r.start..=r.end);
        }
        rle16
    }
}

impl<'a> FromIterator<&'a u16> for RunBlock {
    fn from_iter<I>(iterable: I) -> Self
    where
        I: IntoIterator<Item = &'a u16>,
    {
        let mut run = RunBlock::new();
        for bit in iterable {
            run.insert(*bit);
        }
        run
    }
}

impl FromIterator<ops::Range<u32>> for RunBlock {
    fn from_iter<I>(iterable: I) -> Self
    where
        I: IntoIterator<Item = ops::Range<u32>>,
    {
        let mut weight = 0;
        let mut ranges = Vec::new();
        for curr in iterable {
            assert!(curr.start < curr.end);
            weight += curr.end - curr.start;

            let start = curr.start as u16;
            let end = (curr.end - 1) as u16;

            if ranges.is_empty() {
                ranges.push(start..=end);
                continue;
            }

            let i = ranges.len() - 1;
            assert!(ranges[i].end <= start); // no overlap

            if start == (ranges[i].end + 1) {
                // merge into a previous range
                ranges[i] = ranges[i].start..=end;
            } else {
                ranges.push(start..=end);
            }
        }
        Self { weight, ranges }
    }
}

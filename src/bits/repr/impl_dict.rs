use bits::{PopCount, Rank, Select0, Select1};
use super::{ArrBlock, RunBlock, SeqBlock};

pub const SIZE: u32 = 1 << 16;

impl PopCount<u32> for SeqBlock {
    const SIZE: u32 = SIZE;
    fn count1(&self) -> u32 {
        self.vector.len() as u32
    }
}
impl PopCount<u32> for ArrBlock {
    const SIZE: u32 = SIZE;
    fn count1(&self) -> u32 {
        self.weight
    }
}
impl PopCount<u32> for RunBlock {
    const SIZE: u32 = SIZE;
    fn count1(&self) -> u32 {
        self.weight
    }
}

impl Rank<u16> for SeqBlock {
    fn rank1(&self, i: u16) -> u16 {
        let p = |p| self.vector.get(p).map_or(false, |&v| v >= i);
        search!(0, self.vector.len(), p) as u16
    }
}

impl Rank<u16> for ArrBlock {
    fn rank1(&self, i: u16) -> u16 {
        let (i, p) = divrem!(i, 64);
        let init = self.bitmap.iter().take(i).fold(0, |acc, w| {
            let c1: u16 = w.count1();
            acc + c1
        });
        let last = self.bitmap
            .get(i)
            .map_or(0, |w| w.rank1(u32::from(p)) as u16);
        init + last
    }
}

impl Rank<u16> for RunBlock {
    fn rank1(&self, i: u16) -> u16 {
        match self.search(&i) {
            Ok(n) => {
                let r = self.ranges
                    .iter()
                    .map(|r| r.end - r.start + 1)
                    .take(n)
                    .sum::<u16>();
                i - self.ranges[n].start + r
            }
            Err(n) => if n >= self.ranges.len() {
                self.weight as u16
            } else {
                self.ranges
                    .iter()
                    .map(|r| r.end - r.start + 1)
                    .take(n)
                    .sum::<u16>()
            },
        }
    }
}

impl Select1<u16> for SeqBlock {
    fn select1(&self, c: u16) -> Option<u16> {
        if u32::from(c) >= self.count1() {
            None
        } else {
            self.vector.get(c as usize).cloned()
        }
    }
}

impl Select0<u16> for SeqBlock {
    fn select0(&self, c: u16) -> Option<u16> {
        if u32::from(c) >= self.count0() {
            None
        } else {
            select_by_rank!(0, self, c, 0u32, SIZE, u16)
        }
    }
}

impl Select1<u16> for ArrBlock {
    fn select1(&self, c: u16) -> Option<u16> {
        if u32::from(c) >= self.count1() {
            None
        } else {
            let mut remain = u32::from(c);
            for (i, bit) in self.bitmap.iter().enumerate().filter(|&(_, v)| *v != 0) {
                let ones = bit.count1();
                if remain < ones {
                    let select = bit.select1(remain).unwrap_or(0);
                    return Some((i * 64) as u16 + select as u16);
                }
                remain -= ones;
            }
            None
        }
    }
}

impl Select0<u16> for ArrBlock {
    fn select0(&self, c: u16) -> Option<u16> {
        if u32::from(c) >= self.count0() {
            None
        } else {
            let mut remain = u32::from(c);
            for (i, bit) in self.bitmap.iter().enumerate() {
                let zeros = bit.count0();
                if remain < zeros {
                    let select = bit.select0(remain).unwrap_or(0);
                    return Some((i * 64) as u16 + select as u16);
                }
                remain -= zeros;
            }
            None
        }
    }
}

impl Select1<u16> for RunBlock {
    fn select1(&self, c: u16) -> Option<u16> {
        if u32::from(c) >= self.count1() {
            None
        } else {
            let mut curr = 0;
            for range in &self.ranges {
                let next = curr + (range.end - range.start + 1);
                if next > c {
                    return Some(range.start - curr + c);
                }
                curr = next;
            }
            None
        }
    }
}

impl Select0<u16> for RunBlock {
    fn select0(&self, c: u16) -> Option<u16> {
        if u32::from(c) >= self.count0() {
            None
        } else {
            select_by_rank!(0, self, c, 0u32, SIZE, u16)
        }
    }
}

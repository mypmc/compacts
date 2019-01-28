use crate::{bits::ucast, bits::*};

use super::{Members, Run, BLOCK_SIZE};

use std::cmp;

impl Default for Run {
    fn default() -> Self {
        Self::new()
    }
}

impl Run {
    pub fn new() -> Self {
        Run(Vec::new())
    }

    fn search(&self, n: u16) -> Result<usize, usize> {
        self.0.binary_search_by(|range| {
            if *range.start() <= n && n <= *range.end() {
                cmp::Ordering::Equal
            } else if n < *range.start() {
                cmp::Ordering::Greater
            } else {
                // range.end < n
                cmp::Ordering::Less
            }
        })
    }

    #[inline]
    fn index_to_insert(&self, i: u16) -> Option<usize> {
        self.search(i).err()
    }

    #[inline]
    fn index_to_remove(&self, i: u16) -> Option<usize> {
        self.search(i).ok()
    }

    // fn brackets<'a>(&'a self) -> impl Iterator<Item = Bracket<u16>> + 'a {
    //     self.0
    //         .iter()
    //         .flat_map(|range| vec![Bracket::Start(*range.start()), Bracket::End(*range.end())])
    // }

    // fn bracket(&self, i: u16) -> Option<Bracket<u16>> {
    //     let j = i / 2;
    //     let k = i % 2;
    //     self.0.get(j as usize).map(|range| {
    //         if k == 0 {
    //             Bracket::Start(*range.start())
    //         } else {
    //             Bracket::End(*range.end())
    //         }
    //     })
    // }
}

impl Access for Run {
    fn access(&self, i: u64) -> bool {
        let i = i as u16;
        self.search(i).is_ok()
    }
}

impl FiniteBits for Run {
    const BITS: u64 = BLOCK_SIZE as u64;
    fn empty() -> Self {
        Self::default()
    }
}

impl Count for Run {
    fn bits(&self) -> u64 {
        Self::BITS
    }
    fn count1(&self) -> u64 {
        self.0
            .iter()
            .map(|r| ucast::<u16, u64>(r.end() - r.start()) + 1)
            .sum()
    }
}

impl Rank for Run {
    fn rank1(&self, i: u64) -> u64 {
        let iter = self
            .0
            .iter()
            .map(|r| ucast::<u16, u64>(r.end() - r.start()) + 1);
        let b = i as u16;
        match self.search(b) {
            Ok(n) => iter.take(n).sum::<u64>() + i - u64::from(*self.0[n].start()),
            Err(n) => iter.take(n).sum(),
        }
    }
}

impl Select1 for Run {
    fn select1(&self, c: u64) -> Option<u64> {
        let mut curr = 0;
        for range in &self.0 {
            let next = curr + ucast::<u16, u64>(range.end() - range.start()) + 1;
            if next > c {
                return Some(u64::from(*range.start()) - curr + c);
            }
            curr = next;
        }
        None
    }
}

impl Select0 for Run {
    fn select0(&self, c: u64) -> Option<u64> {
        self.search0(c)
    }
}

impl Assign<u64> for Run {
    type Output = ();
    fn set1(&mut self, i: u64) -> Self::Output {
        let i = i as u16;
        if let Some(pos) = self.index_to_insert(i) {
            let ranges = &mut self.0;

            let lhs = if pos != 0 {
                Some(*ranges[pos - 1].end())
            } else {
                None
            };
            let rhs = if pos < ranges.len() {
                Some(*ranges[pos].start())
            } else {
                None
            };

            match (lhs, rhs) {
                // connect lhs and rhs
                (Some(lhs), Some(rhs)) if lhs + 1 == i && i == rhs - 1 => {
                    let start = *ranges[pos - 1].start();
                    let end = *ranges[pos].end();
                    ranges[pos - 1] = start..=end;
                    ranges.remove(pos);
                }
                // extend lhs
                (Some(lhs), None) if lhs + 1 == i => {
                    let start = *ranges[pos - 1].start();
                    let end = *ranges[pos - 1].end() + 1;
                    ranges[pos - 1] = start..=end;
                }
                // extend rhs
                (None, Some(rhs)) if i == rhs - 1 => {
                    let start = *ranges[pos].start() - 1;
                    let end = *ranges[pos].end();
                    ranges[pos] = start..=end;
                }
                _ => {
                    ranges.insert(pos, i..=i);
                }
            }
        }
    }

    fn set0(&mut self, index: u64) -> Self::Output {
        let index = index as u16;
        if let Some(pos) = self.index_to_remove(index) {
            let ranges = &mut self.0;

            #[allow(clippy::range_minus_one)]
            match (*ranges[pos].start(), *ranges[pos].end()) {
                (i, j) if i == j => {
                    ranges.remove(pos);
                }
                (i, j) if i < index && index < j => {
                    ranges[pos] = i..=(index - 1);
                    ranges.insert(pos + 1, (index + 1)..=j);
                }
                (i, j) if i == index => {
                    assert!(i < j);
                    ranges[pos] = (i + 1)..=j;
                }
                (i, j) if index == j => {
                    assert!(i < j);
                    ranges[pos] = i..=(j - 1);
                }
                _ => unreachable!(),
            };
        }
    }
}

// FIXME: This can be more efficient
impl Assign<std::ops::Range<u64>> for Run {
    type Output = ();
    fn set1(&mut self, index: std::ops::Range<u64>) -> Self::Output {
        for i in index {
            self.set1(i);
        }
    }
    fn set0(&mut self, index: std::ops::Range<u64>) -> Self::Output {
        for i in index {
            self.set0(i);
        }
    }
}

impl std::iter::FromIterator<std::ops::Range<u32>> for Run {
    fn from_iter<I>(iterable: I) -> Self
    where
        I: IntoIterator<Item = std::ops::Range<u32>>,
    {
        let mut ranges = Vec::new();
        for curr in iterable {
            assert!(curr.start < curr.end);

            let start = curr.start as u16;
            let end = (curr.end - 1) as u16; // to inclusive

            // 1st time
            if ranges.is_empty() {
                ranges.push(start..=end);
                continue;
            }

            let i = ranges.len() - 1;
            assert!(*ranges[i].end() <= start); // no overlap

            if start == (ranges[i].end() + 1) {
                // merge into a previous range
                ranges[i] = *ranges[i].start()..=end;
            } else {
                ranges.push(start..=end);
            }
        }
        Run(ranges)
    }
}

impl<'a> std::ops::BitAndAssign<&'a Run> for Run {
    fn bitand_assign(&mut self, that: &'a Run) {
        let members = Members::new(self.0.iter(), that.0.iter());
        *self = members.filter_and().collect();
    }
}

impl<'a> std::ops::BitOrAssign<&'a Run> for Run {
    fn bitor_assign(&mut self, that: &'a Run) {
        let members = Members::new(self.0.iter(), that.0.iter());
        *self = members.filter_or().collect();
    }
}

impl<'a> std::ops::BitXorAssign<&'a Run> for Run {
    fn bitxor_assign(&mut self, that: &'a Run) {
        let members = Members::new(self.0.iter(), that.0.iter());
        *self = members.filter_xor().collect();
    }
}

impl<'a> std::ops::Not for Run {
    type Output = Run;
    fn not(mut self) -> Self::Output {
        let members = Members::new(self.0.iter(), std::iter::empty());
        self = members.filter_not().collect();
        self
    }
}
impl<'a> std::ops::Not for &'a Run {
    type Output = Run;
    fn not(self) -> Self::Output {
        let members = Members::new(self.0.iter(), std::iter::empty());
        members.filter_not().collect()
    }
}

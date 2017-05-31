use std::ops::RangeInclusive;
use std::mem;
use super::{range, Seq16, Seq64, Rle16};
use self::range::Folding;

#[derive(Debug, Default, Clone)]
struct Rle16Builder {
    state: Option<(u16, u16)>, // idx, len

    runlen: usize,
    weight: u32,
    ranges: Vec<RangeInclusive<u16>>,
}

macro_rules! monotonic {
    ( $this:ident, $x:expr, $do:ident ) => {
        if let Some((idx, len)) = $this.state {
            let prev = $this.last_idx(idx, len);
            if $x <= prev {
                // leaped or duplicated
            } else if $x == prev + 1 {
                $this.weight += 1;
                $this.state = Some((idx, len + 1));
            } else {
                $this.$do(idx, len);
                $this.reset($x);
            }
        } else {
            $this.reset($x);
        }
    };
}

macro_rules! packed {
    ( $this:ident, $i:expr, $x:expr, $do:ident ) => {
        {
            const WIDTH: u16 = <u64 as ::UnsignedInt>::WIDTH as u16;
            let mut word = $x;
            for pos in 0..WIDTH {
                if word & (1 << pos) != 0 {
                    let bit = ($i as u16 * WIDTH) + pos;
                    $this.$do(bit);
                    word &= !(1 << pos);
                }
            }
        }
    };
}

impl Rle16Builder {
    fn new() -> Self {
        Self::default()
    }

    fn build(self) -> Rle16 {
        let mut this = self;
        if let Some((idx, len)) = this.state {
            this.push_rle(idx, len);
        }
        Rle16 {
            weight: this.weight,
            ranges: this.ranges,
        }
    }

    fn run(&self) -> usize {
        if self.state.is_some() {
            self.runlen + 1
        } else {
            self.runlen
        }
    }


    fn reset(&mut self, x: u16) {
        self.weight += 1;
        self.state = Some((x, 1));
    }

    fn last_idx(&self, idx: u16, len: u16) -> u16 {
        len - 1 + idx
    }

    // assume sorted values, ignore leaped or duplicated.
    fn incr_monotonic(&mut self, x: u16) {
        monotonic!(self, x, incr_rle);
    }

    fn incr_packed_bits(&mut self, i: usize, x: u64) {
        packed!(self, i, x, incr_monotonic)
    }

    // assume sorted values, ignore leaped or duplicated.
    fn push_monotonic(&mut self, x: u16) {
        monotonic!(self, x, push_rle);
    }

    fn push_packed_bits(&mut self, i: usize, x: u64) {
        packed!(self, i, x, push_monotonic)
    }

    #[inline]
    fn incr_rle(&mut self, _idx: u16, _len: u16) {
        self.runlen += 1;
    }

    #[inline]
    fn push_rle(&mut self, idx: u16, len: u16) {
        let end = self.last_idx(idx, len);
        self.ranges.push(idx...end);
    }
}

impl Seq16 {
    pub fn count_rle(&self) -> usize {
        let mut b = Rle16Builder::new();
        for &bit in &self.vector {
            b.incr_monotonic(bit);
        }
        b.run()
    }
    pub fn mem_in_rle(&self) -> usize {
        let run = self.count_rle();
        run * mem::size_of::<RangeInclusive<u16>>() + mem::size_of::<u32>()
    }
}

impl Seq64 {
    pub fn count_rle(&self) -> usize {
        let mut b = Rle16Builder::new();
        let vec = self.vector.iter().enumerate().filter(|&(_, &v)| v != 0);
        for (i, &bit) in vec {
            b.incr_packed_bits(i, bit);
        }
        b.run()
    }
    pub fn mem_in_rle(&self) -> usize {
        let run = self.count_rle();
        run * mem::size_of::<RangeInclusive<u16>>() + mem::size_of::<u32>()
    }
}

impl Rle16 {
    pub fn count_rle(&self) -> usize {
        self.ranges.len()
    }
    pub fn mem_in_rle(&self) -> usize {
        self.mem()
    }

    fn search(&self, x: u16) -> Result<usize, usize> {
        use std::cmp::Ordering;
        self.ranges
            .binary_search_by(|range| if range.start <= x && x <= range.end {
                                  Ordering::Equal
                              } else if x < range.start {
                                  Ordering::Greater
                              } else if range.end < x {
                                  Ordering::Less
                              } else {
                                  unreachable!()
                              })
    }

    fn index_to_insert(&self, x: u16) -> Option<usize> {
        self.search(x).err()
    }
    fn index_to_remove(&self, x: u16) -> Option<usize> {
        self.search(x).ok()
    }

    pub fn contains(&self, x: u16) -> bool {
        self.search(x).is_ok()
    }

    pub fn insert(&mut self, x: u16) -> bool {
        let pos = self.index_to_insert(x);
        if pos.is_none() {
            return false;
        }
        let pos = pos.unwrap();
        self.weight += 1;

        let lhs = if pos > 0 && pos <= self.ranges.len() {
            Some(self.ranges[pos - 1].end)
        } else {
            None
        };
        let rhs = if pos < (::std::u16::MAX as usize) && pos < self.ranges.len() {
            Some(self.ranges[pos].start)
        } else {
            None
        };

        match (lhs, rhs) {
            (None, Some(rhs)) if x == rhs - 1 => {
                self.ranges[pos] = (self.ranges[pos].start - 1)...self.ranges[pos].end;
            }
            (Some(lhs), Some(rhs)) if lhs + 1 == x && x == rhs - 1 => {
                let i = pos - 1;
                self.ranges[i] = self.ranges[i].start...self.ranges[pos].end;
                self.ranges.remove(pos);
            }
            (Some(lhs), _) if lhs + 1 == x => {
                let i = pos - 1;
                self.ranges[i] = self.ranges[i].start...(self.ranges[i].end + 1);
            }
            (_, Some(rhs)) if x == rhs - 1 => {
                self.ranges[pos] = (self.ranges[pos].start - 1)...self.ranges[pos].end;
            }
            _ => {
                self.ranges.insert(pos, x...x);
            }
        }
        return true;
    }

    pub fn remove(&mut self, x: u16) -> bool {
        let pos = self.index_to_remove(x);
        if pos.is_none() {
            return false;
        }
        let pos = pos.unwrap();
        self.weight -= 1;

        match (self.ranges[pos].start, self.ranges[pos].end) {
            (i, j) if i == j => {
                self.ranges.remove(pos);
            }
            (i, j) if i < x && x < j => {
                self.ranges.remove(pos);
                let idx = pos as u16;
                self.ranges.insert(pos, i...(idx - 1));
                self.ranges.insert(pos + 1, (idx + 1)...j);
            }
            (i, j) if i == x => {
                self.ranges[pos] = (i + 1)...j;
                debug_assert!((i + 1) <= j);
            }
            (i, j) if j == x => {
                self.ranges[pos] = i...(j - 1);
                debug_assert!(i <= (j - 1));
            }
            _ => unreachable!(),
        };
        return true;
    }
}

impl From<Seq16> for Rle16 {
    fn from(vec16: Seq16) -> Self {
        Rle16::from(&vec16)
    }
}
impl<'a> From<&'a Seq16> for Rle16 {
    fn from(vec16: &'a Seq16) -> Self {
        vec16.vector.iter().collect()
    }
}

impl From<Seq64> for Rle16 {
    fn from(vec64: Seq64) -> Self {
        Rle16::from(&vec64)
    }
}
impl<'a> From<&'a Seq64> for Rle16 {
    fn from(vec64: &'a Seq64) -> Self {
        let mut b = Rle16Builder::new();
        let vec = vec64.vector.iter().enumerate().filter(|&(_, &v)| v != 0);
        for (i, &bit) in vec {
            b.push_packed_bits(i, bit);
        }
        b.build()
    }
}

impl<'a> ::std::iter::FromIterator<&'a u16> for Rle16 {
    fn from_iter<I>(iterable: I) -> Self
        where I: IntoIterator<Item = &'a u16>
    {
        let mut b = Rle16Builder::new();
        for &bit in iterable {
            b.push_monotonic(bit);
        }
        b.build()
    }
}

impl<'a> From<&'a [RangeInclusive<u16>]> for Rle16 {
    fn from(slice: &'a [RangeInclusive<u16>]) -> Self {
        let mut rle16 = Rle16 {
            weight: 0,
            ranges: Vec::with_capacity(slice.len()),
        };
        for r in slice {
            let w = (r.end - r.start) + 1;
            rle16.weight += w as u32;
            rle16.ranges.push(r.start...r.end);
        }
        rle16
    }
}

impl<'a, 'b> ::ops::Intersection<&'b Rle16> for &'a Rle16 {
    type Output = Rle16;
    fn intersection(self, rle16: &'b Rle16) -> Self::Output {
        let chunks = Folding::new(&self.ranges, &rle16.ranges).intersection();
        let (weight, ranges) = range::repair(chunks);
        Rle16 { weight, ranges }
    }
}

impl<'a, 'b> ::ops::Union<&'b Rle16> for &'a Rle16 {
    type Output = Rle16;
    fn union(self, rle16: &'b Rle16) -> Self::Output {
        let chunks = Folding::new(&self.ranges, &rle16.ranges).union();
        let (weight, ranges) = range::repair(chunks);
        Rle16 { weight, ranges }
    }
}

impl<'a, 'b> ::ops::Difference<&'b Rle16> for &'a Rle16 {
    type Output = Rle16;
    fn difference(self, rle16: &'b Rle16) -> Self::Output {
        let chunks = Folding::new(&self.ranges, &rle16.ranges).difference();
        let (weight, ranges) = range::repair(chunks);
        Rle16 { weight, ranges }
    }
}

impl<'a, 'b> ::ops::SymmetricDifference<&'b Rle16> for &'a Rle16 {
    type Output = Rle16;
    fn symmetric_difference(self, rle16: &'b Rle16) -> Self::Output {
        let chunks = Folding::new(&self.ranges, &rle16.ranges).symmetric_difference();
        let (weight, ranges) = range::repair(chunks);
        Rle16 { weight, ranges }
    }
}


#[cfg(test)]
mod rle16_tests {
    use super::*;

    static RLE: &[::std::ops::RangeInclusive<u16>] = &[1...1, 3...5, 10...13, 18...19, 100...120];

    #[test]
    fn index_to_insert() {
        let rle = Rle16::from(RLE);
        assert_eq!(rle.index_to_insert(0), Some(0));
        assert_eq!(rle.index_to_insert(2), Some(1));
        assert_eq!(rle.index_to_insert(3), None);
        assert_eq!(rle.index_to_insert(5), None);
        assert_eq!(rle.index_to_insert(6), Some(2));
        assert_eq!(rle.index_to_insert(8), Some(2));
        assert_eq!(rle.index_to_insert(16), Some(3));
        assert_eq!(rle.index_to_insert(80), Some(4));
        assert_eq!(rle.index_to_insert(200), Some(5));
    }

    #[test]
    fn index_to_remove() {
        let rle = Rle16::from(RLE);
        assert_eq!(rle.index_to_remove(0), None);
        assert_eq!(rle.index_to_remove(2), None);
        assert_eq!(rle.index_to_remove(3), Some(1));
        assert_eq!(rle.index_to_remove(5), Some(1));
        assert_eq!(rle.index_to_remove(6), None);
        assert_eq!(rle.index_to_remove(8), None);
        assert_eq!(rle.index_to_remove(18), Some(3));
        assert_eq!(rle.index_to_remove(110), Some(4));
        assert_eq!(rle.index_to_remove(200), None);
    }

    #[test]
    fn rle16_insert() {
        let slice = [1...1, 3...5];
        let mut rle = Rle16::from(&slice[..]);

        rle.insert(2);
        assert_eq!(rle.count_ones(), 5);
        assert_eq!(rle.ranges, &[1...5]);

        rle.insert(8);
        assert_eq!(rle.count_ones(), 6);
        assert_eq!(rle.ranges, &[1...5, 8...8]);

        rle.insert(10);
        assert_eq!(rle.count_ones(), 7);
        assert_eq!(rle.ranges, &[1...5, 8...8, 10...10]);

        rle.insert(7);
        assert_eq!(rle.count_ones(), 8);
        assert_eq!(rle.ranges, &[1...5, 7...8, 10...10]);

        rle.insert(9);
        assert_eq!(rle.count_ones(), 9);
        assert_eq!(rle.ranges, &[1...5, 7...10]);

        rle.insert(6);
        assert_eq!(rle.count_ones(), 10);
        assert_eq!(rle.ranges, &[1...10]);

        let max = ::std::u16::MAX;
        rle.insert(max);
        assert_eq!(rle.count_ones(), 11);
        assert_eq!(rle.ranges, &[1...10, max...max]);
    }

    #[test]
    fn rle16_remove() {
        let slice = [1...1, 3...5];
        let mut rle = Rle16::from(&slice[..]);
        assert_eq!(rle.count_ones(), 4);
        rle.remove(0);
        assert_eq!(rle.count_ones(), 4);
        assert_eq!(rle.ranges, &[1...1, 3...5]);
        rle.remove(1);
        assert_eq!(rle.count_ones(), 3);
        assert_eq!(rle.ranges, &[3...5]);
        rle.remove(2);
        assert_eq!(rle.count_ones(), 3);
        assert_eq!(rle.ranges, &[3...5]);
        rle.remove(5);
        assert_eq!(rle.count_ones(), 2);
        assert_eq!(rle.ranges, &[3...4]);
        rle.remove(3);
        assert_eq!(rle.count_ones(), 1);
        assert_eq!(rle.ranges, &[4...4]);
        rle.remove(4);
        assert_eq!(rle.count_ones(), 0);
        assert_eq!(rle.ranges, &[]);
    }
}

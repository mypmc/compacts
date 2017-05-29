use std::ops::RangeInclusive;
use super::{range, Seq16, Seq64, Rle16};
use self::range::Folding;
use Select1;

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
            while let Some(pos) = word.select1(0) {
                let bit = ($i as u16 * WIDTH) + pos;
                $this.$do(bit);
                word &= !(1 << pos);
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
}

impl Rle16 {
    pub fn count_rle(&self) -> usize {
        self.ranges.len()
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

use std::ops::RangeInclusive;
use super::{Seq16, Seq64, Rle16};
use Select1;

#[derive(Debug, Default, Clone)]
struct Rle16Builder {
    state: Option<(u16, u16)>, // idx, len

    weight: u32,
    ranges: Vec<RangeInclusive<u16>>,
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

    fn reset(&mut self, x: u16) {
        self.weight += 1;
        self.state = Some((x, 1));
    }

    fn last_idx(&self, idx: u16, len: u16) -> u16 {
        len - 1 + idx
    }

    // assume sorted values, ignore leaped or duplicated.
    fn push_monotonic(&mut self, x: u16) {
        if let Some((idx, len)) = self.state {
            let prev = self.last_idx(idx, len);
            if x <= prev {
                // leaped or duplicated
            } else if x == prev + 1 {
                self.weight += 1;
                self.state = Some((idx, len + 1));
            } else {
                self.push_rle(idx, len);
                self.reset(x);
            }
        } else {
            self.reset(x);
        }
    }

    fn push_packed_bits(&mut self, i: usize, x: u64) {
        const WIDTH: u16 = <u64 as ::UnsignedInt>::WIDTH as u16;
        let mut word = x;
        while let Some(pos) = word.select1(0) {
            let bit = (i as u16 * WIDTH) + pos;
            self.push_monotonic(bit);
            word &= !(1 << pos);
        }
    }

    fn push_rle(&mut self, idx: u16, len: u16) {
        let end = self.last_idx(idx, len);
        self.ranges.push(idx...end);
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

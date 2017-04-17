use super::{Bits, Repr, Rank0, Rank1};

impl Rank0 for Repr {
    fn rank0(&self, i: usize) -> usize {
        i - self.rank1(i)
    }
}

impl Rank1 for Repr {
    fn rank1(&self, i: usize) -> usize {
        if i >= Self::SIZE {
            return self.ones();
        }
        let rank = match self {
            &Repr::Vec(_, ref bits) => {
                let j = i as u16;
                match bits.binary_search(&j) {
                    Err(r) if r > bits.len() => self.ones(), // rank - 1
                    Err(r) | Ok(r) => r,
                }
            }
            &Repr::Map(_, ref bits) => {
                let q = i / Self::BITS_SIZE;
                let r = i % Self::BITS_SIZE;
                bits.iter().take(q).fold(0, |acc, w| acc + w.ones()) +
                bits.get(q).map_or(0, |w| w.rank1(r))
            }
        };
        return rank;
    }
}

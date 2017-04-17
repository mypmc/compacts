use super::{Bits, Repr, Rank0, Rank1};

impl Rank0<usize> for Repr {
    fn rank0(&self, i: usize) -> u64 {
        i as u64 - self.rank1(i)
    }
}

impl Rank1<usize> for Repr {
    fn rank1(&self, i: usize) -> u64 {
        if i as u64 >= Self::SIZE {
            return self.ones();
        }
        let rank = match self {
            &Repr::Vec(_, ref bits) => {
                let j = i as u16;
                match bits.binary_search(&j) {
                    Err(r) if r > bits.len() => self.ones(), // rank - 1
                    Err(r) | Ok(r) => r as u64,
                }
            }
            &Repr::Map(_, ref bits) => {
                let q = i / Self::BITS_SIZE as usize;
                let r = i % Self::BITS_SIZE as usize;
                bits.iter().take(q).fold(0, |acc, w| acc + w.ones()) +
                bits.get(q).map_or(0, |w| w.rank1(r))
            }
        };
        return rank;
    }
}

use UnsignedInt;

pub trait Rank<T: UnsignedInt> {
    /// Hamming Weight or Population Count.
    type Weight;

    const SIZE: Self::Weight;

    /// Returns occurences of non-zero bit in `0...i`.
    /// It's equivalent to `i+1 - self.rank0(i)`.
    fn rank1(&self, i: T) -> Self::Weight;

    /// Returns occurences of zero bit in `0...i`.
    /// It's equivalent to `i+1 - self.rank1(i)`.
    fn rank0(&self, i: T) -> Self::Weight;
}

impl Rank<u32> for u64 {
    type Weight = u32;

    const SIZE: Self::Weight = <u64 as ::UnsignedInt>::WIDTH as u32;

    fn rank1(&self, i: u32) -> Self::Weight {
        if Self::Weight::from(i).succ() >= <u64 as ::UnsignedInt>::WIDTH as u32 {
            self.count_ones()
        } else {
            let mask = (1 << (i + 1)) - 1;
            (self & mask).count_ones()
        }
    }

    fn rank0(&self, i: u32) -> Self::Weight {
        i + 1 - self.rank1(i)
    }
}

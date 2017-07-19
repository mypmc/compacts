use UnsignedInt;

/// Rank is a generalization of `PopCount` or `HammingWeight`.
pub trait Rank<Index: UnsignedInt> {
    /// This type should be large enough to count 0/1,
    /// even if all bit are set or not.
    type Count = Index;

    /// Returns occurences of non-zero bit in `[0, i]`.
    ///   - `rank1(i)` should be equal to `i + 1 - self.rank0(i)`.
    ///   - `rank1(i::MAX_BOUND)` should be equal to `count_ones()`,
    fn rank1(&self, i: Index) -> Self::Count;

    /// Returns occurences of zero bit in `[0, i]`.
    ///   - `rank0(i)` should be equal to `i + 1 - self.rank1(i)`.
    ///   - `rank0(i::MAX_BOUND)` should be equal to `count_zeros()`,
    fn rank0(&self, i: Index) -> Self::Count;
}

impl Rank<u32> for u64 {
    type Count = u32;
    fn rank1(&self, i: u32) -> Self::Count {
        if i + 1 >= <u64 as UnsignedInt>::WIDTH as u32 {
            self.count_ones()
        } else {
            let mask = (1 << (i + 1)) - 1;
            (self & mask).count_ones()
        }
    }

    fn rank0(&self, i: u32) -> Self::Count {
        i + 1 - self.rank1(i)
    }
}

/// Rank is a generalization of `PopCount` or `HammingWeight`.
pub trait Rank<Index> {
    type Count = Index;

    /// Returns occurences of non-zero bit in `[0, i)` for `0 < i`.
    /// `rank1(i)` should be equal to `i - self.rank0(i)`.
    fn rank1(&self, i: Index) -> Self::Count;

    /// Returns occurences of zero bit in `[0, i)` for `0 < i`.
    /// `rank0(i)` should be equal to `i - self.rank1(i)`.
    fn rank0(&self, i: Index) -> Self::Count;
}

impl Rank<u32> for u64 {
    type Count = u32;

    fn rank1(&self, i: u32) -> Self::Count {
        // assert!(i > 0);
        if i == 0 {
            return 0;
        }
        if i >= 64 as u32 {
            self.count_ones()
        } else {
            let mask = (1 << i) - 1;
            (self & mask).count_ones()
        }
    }

    fn rank0(&self, i: u32) -> Self::Count {
        // assert!(i > 0);
        if i == 0 {
            return 0;
        }
        i - self.rank1(i)
    }
}

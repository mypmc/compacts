use std::ops;

/// PopCount is a trait for `PopCount` or `HammingWeight`.
pub trait PopCount<T>
where
    T: ops::Sub<Output = T> + Copy,
{
    const SIZE: T;

    fn count1(&self) -> T {
        Self::SIZE - self.count0()
    }

    fn count0(&self) -> T {
        Self::SIZE - self.count1()
    }
}

/// Rank is a generalization of `PopCount`.
pub trait Rank<T>
where
    T: ops::Sub<Output = T> + Copy,
{
    /// Returns occurences of non-zero bit in `[0, i)` for `0 < i`.
    /// `rank1(i)` should be equal to `i - self.rank0(i)`.
    fn rank1(&self, i: T) -> T {
        i - self.rank0(i)
    }

    /// Returns occurences of zero bit in `[0, i)` for `0 < i`.
    /// `rank0(i)` should be equal to `i - self.rank1(i)`.
    fn rank0(&self, i: T) -> T {
        i - self.rank1(i)
    }
}

macro_rules! impl_PopCount {
    ( $( $out:ty ),* ) => ($(
        impl PopCount<$out> for u64 {
            const SIZE: $out = 64;
            fn count1(&self) -> $out {
                self.count_ones() as $out
            }
        }
    )*)
}
impl_PopCount!(u64, u32, u16, u8);

impl Rank<u32> for u64 {
    fn rank1(&self, i: u32) -> u32 {
        if i == 0 {
            return 0;
        }
        if i >= <u64 as PopCount<u32>>::SIZE {
            self.count1()
        } else {
            let mask = (1 << i) - 1;
            (self & mask).count1()
        }
    }
}

use prim::UnsignedInt;

pub trait Rank<T: ::UnsignedInt> {
    // `From<T>` constrain that Weight should be able to construct from T safely,
    // because Weight may hold a value that is greater than or equal to `T::max_value() + 1`.

    /// Hamming Weight or Population Count.
    type Weight: ::UnsignedInt;

    fn size(&self) -> Self::Weight
        where Self::Weight: From<T>
    {
        self.rank1(T::max_bound()) + self.rank0(T::max_bound())
    }

    /// Returns occurences of non-zero bit in `0...i`.
    /// It's equivalent to `i+1 - self.rank0(i)`.
    fn rank1(&self, i: T) -> Self::Weight
        where Self::Weight: From<T>
    {
        // i+1 may overflow, so first convert to Self::Weight
        Self::Weight::from(i).succ() - self.rank0(i)
    }

    /// Returns occurences of zero bit in `0...i`.
    /// It's equivalent to `i+1 - self.rank1(i)`.
    fn rank0(&self, i: T) -> Self::Weight
        where Self::Weight: From<T>
    {
        Self::Weight::from(i).succ() - self.rank1(i)
    }
}

macro_rules! impl_Rank {
    ( $( $index:ty ),* ) => ($(
        impl Rank<$index> for u64 {
            type Weight = u32;

            fn size(&self) -> Self::Weight { <u64 as ::UnsignedInt>::WIDTH as u32 }

            fn rank1(&self, i: $index) -> Self::Weight {
                if Self::Weight::from(i).succ() >= <u64 as ::UnsignedInt>::WIDTH as u32 {
                    self.count_ones()
                } else {
                    let mask = (1 << (i + 1)) - 1;
                    (self & mask).count_ones()
                }
            }
        }
    )*)
}
impl_Rank!(u32); // make compiler infer type

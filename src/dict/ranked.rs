use super::prim::Uint;

/// Ranked is a dict for bit.
/// Type that implement `Index<T, Output = bool> + Ranked + Select0 + Select1`
/// can be Dict automatically.
///
/// You MUST implement
///
/// - `size`
///
/// - at least one of `count1` and `count0`.
///
/// - at least one of `rank1` and `rank0`.
///
/// Otherwise, stack may overflow because of cyclic default definitions.
pub trait Ranked<T: Uint> {
    /// Hamming Weight or Population Count.
    ///
    /// `From<T>` constrain that Weight should be able to construct from T safely,
    /// because Weight may hold a value that is greater than or equal to `T::max_value() + 1`.
    type Weight: Uint + From<T>;

    /// `count1()` + `count0()` **SHOULD** be equal to `size`.
    fn size(&self) -> Self::Weight;
    //fn size(&self) -> Self::Weight { self.count0() + self.count1() }

    /// Count occurences of non-zero bit, **SHOULD** be equal to `rank1(size)`.
    fn count1(&self) -> Self::Weight {
        self.size() - self.count0()
    }
    /// Count occurences of zero bit, **SHOULD** be equal to `rank0(size)`.
    fn count0(&self) -> Self::Weight {
        self.size() - self.count1()
    }

    /// Generalization of `count1`, returns occurences of non-zero bit in `0...i`.
    /// It's equivalent to `i+1 - self.rank0(i)`.
    fn rank1(&self, i: T) -> Self::Weight {
        // i+1 may overflow, so first convert to Self::Weight
        Self::Weight::from(i).succ() - self.rank0(i)
    }
    /// Generalization of `count0`, returns occurences of zero bit in `0...i`.
    /// It's equivalent to `i+1 - self.rank1(i)`.
    fn rank0(&self, i: T) -> Self::Weight {
        Self::Weight::from(i).succ() - self.rank1(i)
    }
}

macro_rules! impl_Ranked_for_Bits {
    ( $( $index:ty ),* ) => ($(
        impl Ranked<$index> for u64 {
            type Weight = u32;

            fn size(&self) -> Self::Weight { <u64 as Uint>::WIDTH as u32 }

            fn count1(&self) -> Self::Weight { self.count_ones() }

            fn rank1(&self, i: $index) -> Self::Weight {
                if Self::Weight::from(i).succ() >= <u64 as Uint>::WIDTH as u32 {
                    self.count_ones()
                } else {
                    let mask = (1 << (i + 1)) - 1;
                    (self & mask).count_ones()
                }
            }
        }

        impl Ranked<$index> for super::Bits {
            type Weight = <u64 as Ranked<$index>>::Weight;

            fn size(&self) -> Self::Weight { <u64 as Uint>::WIDTH as u32 }

            fn count1(&self) -> Self::Weight { (self.0).count_ones() }

            fn rank1(&self, i: $index) -> Self::Weight { (self.0).rank1(i) }
        }
    )*)
}
impl_Ranked_for_Bits!(u32); // make compiler infer type

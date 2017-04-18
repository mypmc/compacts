use super::Bits;

pub trait Rank0<T = usize> {
    /// Count how many zero bits there are up to a given position
    fn rank0(&self, i: T) -> u64;
}
pub trait Rank1<T = usize> {
    /// Count how many non-zero bits there are up to a given position
    fn rank1(&self, i: T) -> u64;
}

macro_rules! impl_rank9 {
    ( $( ($type: ty, $key: ty) ),* ) => ($(
        impl Rank0<$key> for $type {
            #[inline]
            fn rank0(&self, i: $key) -> u64 {
                let rank1 = self.rank1(i);
                i as u64 - rank1
            }
        }
        impl Rank1<$key> for $type {
            #[inline]
            fn rank1(&self, i: $key) -> u64 {
                let rank = if i as u64 >= Self::SIZE {
                    self.ones()
                } else {
                    let this = *self;
                    (this & ((1 << i) - 1)).ones()
                };
                rank
            }
        }
    )*)
}
macro_rules! impl_rank9_all {
    ( $( $type: ty ),* ) => ($(
        impl_rank9!(($type, u64), ($type, u32), ($type, u16), ($type, u8), ($type, usize));
    )*)
}
impl_rank9_all!(u64, u32, u16, usize);

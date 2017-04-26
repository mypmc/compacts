use std::ops::BitAndAssign;
use std::ops::BitOrAssign;
use std::ops::BitXorAssign;
use std::ops::SubAssign;

pub trait Pairwise<Rhs = Self> {
    type Output;

    fn intersection(&self, that: Rhs) -> Self::Output;

    fn union(&self, that: Rhs) -> Self::Output;

    fn difference(&self, that: Rhs) -> Self::Output;

    fn symmetric_difference(&self, that: Rhs) -> Self::Output;
}

pub trait PairwiseWith<Rhs = Self> {
    fn intersection_with(&mut self, that: Rhs);

    fn union_with(&mut self, that: Rhs);

    fn difference_with(&mut self, that: Rhs);

    fn symmetric_difference_with(&mut self, that: Rhs);
}

impl<T> BitAndAssign<T> for PairwiseWith<T> {
    fn bitand_assign(&mut self, that: T) {
        self.intersection_with(that)
    }
}

impl<T> BitOrAssign<T> for PairwiseWith<T> {
    fn bitor_assign(&mut self, that: T) {
        self.union_with(that)
    }
}

impl<T> SubAssign<T> for PairwiseWith<T> {
    fn sub_assign(&mut self, that: T) {
        self.difference_with(that);
    }
}

impl<T> BitXorAssign<T> for PairwiseWith<T> {
    fn bitxor_assign(&mut self, that: T) {
        self.symmetric_difference_with(that);
    }
}

macro_rules! impl_pairwise {
    ( $( $type:ty ),* ) => ($(
        impl PairwiseWith for $type {
            fn intersection_with(&mut self, rhs: $type)         {*self &=  rhs;}
            fn union_with(&mut self, rhs: $type)                {*self |=  rhs;}
            fn difference_with(&mut self, rhs: $type)           {*self &= !rhs;}
            fn symmetric_difference_with(&mut self, rhs: $type) {*self ^=  rhs;}
        }
    )*);
}
impl_pairwise!(u8, u16, u32, u64, usize);

use std::ops;

pub use self::ops::{BitAndAssign, BitOrAssign, BitXorAssign};

pub trait BitAndNotAssign<RHS = Self> {
    fn bitandnot_assign(&mut self, that: RHS);
}

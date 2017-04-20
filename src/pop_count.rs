/// Constant sized bits.
pub trait PopCount {
    /// Max bits size of this representation.
    /// ones() + zeros() == CAPACITY
    const CAPACITY: u64;

    /// Count non-zero bits.
    // REQUIRES: ones() <= Self::CAPACITY
    fn ones(&self) -> u64 {
        Self::CAPACITY - self.zeros()
    }

    /// Count zero bits.
    // REQUIRES: zeros() <= Self::CAPACITY
    fn zeros(&self) -> u64 {
        Self::CAPACITY - self.ones()
    }
}

/// Utility trait for internal use.
pub trait Bounded {
    const MIN: Self;
    const MAX: Self;
}

macro_rules! impl_PopCount {
    ( $( ($type: ty, $size: expr) ),* ) => ($(
        impl Bounded for $type {
            const MIN: $type =  0;
            const MAX: $type = !0;
        }

        impl PopCount for $type {
            const CAPACITY: u64 = $size;
            #[inline] fn ones(&self) -> u64 {
                let ones = self.count_ones() as u64;
                debug_assert!(ones <= Self::CAPACITY);
                ones
            }
        }
    )*)
}
impl_PopCount!((u64, 64), (u32, 32), (u16, 16), (u8, 8));
#[cfg(target_pointer_width = "32")]
impl_PopCount!{(usize, 32)}
#[cfg(target_pointer_width = "64")]
impl_PopCount!{(usize, 64)}

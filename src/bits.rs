// Constant sized bits.
pub trait Bits {
    /// Size of this representation.
    const SIZE: u64;

    /// The value with all bits unset.
    fn none() -> Self;

    /// Count non-zero bits.
    // REQUIRES: ones() <= Self::SIZE
    fn ones(&self) -> u64 {
        Self::SIZE - self.zeros()
    }

    /// Count zero bits.
    // REQUIRES: zeros() <= Self::SIZE
    fn zeros(&self) -> u64 {
        Self::SIZE - self.ones()
    }
}

macro_rules! impl_bits {
    ( $( ($type: ty, $size: expr) ),* ) => ($(
        impl Bits for $type {
            const SIZE: u64 = $size;
            #[inline] fn none() -> Self { 0 }
            #[inline] fn ones(&self) -> u64 {
                let ones = self.count_ones();
                debug_assert!(ones as u64 <= Self::SIZE);
                ones as u64
            }
        }
    )*)
}
impl_bits!((u64, 64), (u32, 32), (u16, 16), (u8, 8));
#[cfg(target_pointer_width = "32")]
impl_bits!{(usize, 32)}
#[cfg(target_pointer_width = "64")]
impl_bits!{(usize, 64)}

impl Bits for bool {
    const SIZE: u64 = 1;
    fn none() -> Self {
        false
    }
    fn ones(&self) -> u64 {
        if *self { 1 } else { 0 }
    }
}

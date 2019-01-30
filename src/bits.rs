//! Module bits defines traits and types to interact with a bits container.
//!
//! ## `Map<T>` and `[T]`
//!
//! ## `EntryMap<K, V>` and `[Entry<K, V>]`

// # References
//
// - Compact Data Structures: A Practical Approach
// - Fast, Small, Simple Rank/Select on Bitmaps
// - Space-Efficient, High-Performance Rank & Select Structures on Uncompressed Bit Sequences

#[macro_use]
mod macros;
#[cfg(test)]
mod tests;

#[allow(dead_code)]
pub mod rrr15 {
    generate_rrr_mod!("/table15.rs", u16, 15, 4);
}
#[allow(dead_code)]
pub mod rrr31 {
    generate_rrr_mod!("/table31.rs", u32, 31, 5);
}
#[allow(dead_code)]
pub mod rrr63 {
    generate_rrr_mod!("/table63.rs", u64, 63, 6);
}

// mod flip;
// mod range;

pub mod ops;

mod block;
mod entry;

mod flat;
mod mask;
mod uint;

pub use self::{
    block::Block,
    entry::{Entry, EntryMap},
    flat::Map,
};

pub use self::mask::{and, or, xor, Fold, Mask};

use self::{
    block::BlockArray,
    ops::*,
    uint::{TryCast, UnsignedInt},
};

const MAX_BITS: u64 = 1 << 63;

// Panic message.
static OUT_OF_BOUNDS: &str = "index out of bounds";

/// Cast U into T.
///
/// # Panics
///
/// Panics if given `u` does not fit in `T`.
#[inline]
fn ucast<U, T>(u: U) -> T
where
    U: UnsignedInt + TryCast<T>,
    T: UnsignedInt,
{
    u.try_cast().expect("does not fit in T")
}

#[inline]
fn divmod<U: UnsignedInt>(i: u64, cap: u64) -> (U, u64)
where
    u64: TryCast<U>,
{
    (ucast(i / cap), i % cap)
}

macro_rules! implMask {
    ( $([ $($constraints:tt)*] for $Type:ty ;)+ ) => {
        $(
            impl<$($constraints)*> $Type {
                pub fn and<Rhs>(&self, rhs: Rhs) -> mask::And<&Self, Rhs> {
                    and(self, rhs)
                }
                pub fn or<Rhs>(&self, rhs: Rhs) -> mask::Or<&Self, Rhs> {
                    or(self, rhs)
                }
                pub fn xor<Rhs>(&self, rhs: Rhs) -> mask::Xor<&Self, Rhs> {
                    xor(self, rhs)
                }

                // pub fn not(&self) -> Flip<&$Type> {
                //     let bits = self.bits();
                //     let data = self;
                //     Flip { bits, data }
                // }
            }
        )+
    }
}
implMask!(
    [T: FiniteBits] for Map<T>;
    [K: UnsignedInt, V: FiniteBits] for EntryMap<K, V>;
);

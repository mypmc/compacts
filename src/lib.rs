// #![deny(warnings)]

#[cfg(test)]
#[macro_use]
extern crate quickcheck;
#[cfg(test)]
#[macro_use]
extern crate lazy_static;
#[cfg(test)]
extern crate rand;

mod private {
    use std::ops;
    pub trait Sealed {}

    macro_rules! impl_Sealed {
        ( $( $Type:ty ),* ) => {
            $( impl Sealed for $Type {} )*
        }
    }
    impl_Sealed!(
        u8,
        u16,
        u32,
        u64,
        u128,
        usize,
        ops::Range<u32>,
        ops::Range<u64>,
        ops::RangeTo<u32>,
        ops::RangeTo<u64>,
        ops::RangeFrom<u32>,
        ops::RangeFrom<u64>,
        ops::RangeFull
    );
}

pub mod bits;

// pub trait Count<T> {
//     type Symbol;
//     /// Returns occurences of symbol `c` in self.
//     fn count(&self, c: &Self::Symbol) -> T;
// }

// /// Generalization of `count`.
// pub trait Rank<T> {
//     type Symbol;
//     /// Returns occurences of symbol `c` in `0..i`.
//     fn rank(&self, c: &Self::Symbol, i: T) -> T;
// }

// /// `Select` is a right inverse of `Rank`.
// pub trait Select<T> {
//     type Symbol;
//     /// Returns the position of 'i+1'th appearance of symbol `c`.
//     fn select(&self, c: &Self::Symbol, i: T) -> Option<T>;
// }

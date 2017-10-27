#![feature(conservative_impl_trait)]
#![feature(inclusive_range)]
#![feature(inclusive_range_syntax)]
// #![deny(warnings)]

extern crate byteorder;
extern crate itertools;
#[cfg(test)]
#[macro_use]
extern crate quickcheck;

mod io;

pub mod bits;

pub use io::{ReadFrom, WriteTo};

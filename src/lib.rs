#![feature(conservative_impl_trait)]
#![feature(inclusive_range)]
#![feature(inclusive_range_syntax)]
#![cfg_attr(test, feature(plugin))]

// #![deny(warnings)]

extern crate byteorder;
extern crate itertools;
#[cfg(test)]
#[macro_use]
extern crate quickcheck;

pub mod bits;

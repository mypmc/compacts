#![feature(i128_type)]
#![feature(integer_atomics)]
#![feature(associated_type_defaults)]
#![feature(conservative_impl_trait)]
#![feature(inclusive_range)]
#![feature(inclusive_range_syntax)]
#![feature(fnbox)]
#![cfg_attr(test, feature(plugin))]

// #![deny(warnings)]

extern crate itertools;
extern crate parking_lot;
#[cfg(test)]
#[macro_use]
extern crate quickcheck;

pub mod bits;
pub mod dict;

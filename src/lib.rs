#![feature(associated_consts)]
#![cfg_attr(test, feature(test))]

/// Broadword implementation of rank/select queries
/// (http://sux.di.unimi.it/paper.pdf);
/// Springer Berlin Heidelberg, 2008. 154-168.
///

extern crate num;
#[macro_use]
extern crate log;
#[macro_use]
extern crate thunk;

pub mod prim;
pub mod bits;
pub mod dict;

pub mod bit_vec;
pub use bit_vec::BitVec;

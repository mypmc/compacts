#![feature(associated_consts)]

#[macro_use]
extern crate karabiner;

extern crate num;

// Broadword implementation of rank/select queries
// (http://sux.di.unimi.it/paper.pdf);
// Springer Berlin Heidelberg, 2008. 154-168.

pub mod dict;

pub mod bits;

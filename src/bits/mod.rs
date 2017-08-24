// References:
//   - Broadword implementation of rank/select queries
//     ( http://sux.di.unimi.it/paper.pdf );
//
//   - Roaring BitMap
//     ( https://arxiv.org/pdf/1603.06549.pdf );

#[macro_use]
mod macros;
#[macro_use]
mod thunk;
mod prim;
mod pair;
mod block;
mod map16;
mod map32;
mod map64;
#[cfg(test)]
mod tests;

pub use self::pair::*;
pub use self::map16::Map16;
pub use self::map32::Map32;
pub use self::map64::Map64;

static TRUE: &bool = &true;
static FALSE: &bool = &false;

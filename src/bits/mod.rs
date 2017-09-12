#[macro_use]
mod macros;
mod prim;
mod io;
mod dict;
mod pair;
mod block;
mod map32;
#[cfg(test)]
mod tests;

pub(crate) use self::block::Block;
pub(crate) use self::prim::{Merge, Split};
pub(crate) use self::pair::{Assign, Compare, Entries, Entry};
pub(crate) static TRUE: &bool = &true;
pub(crate) static FALSE: &bool = &false;

pub use self::io::{ReadFrom, WriteTo};
pub use self::pair::{and, and_not, or, xor, And, AndNot, Or, Xor};
pub use self::dict::{/* Dict, */ PopCount, Rank, Select0, Select1};
pub use self::map32::Map;

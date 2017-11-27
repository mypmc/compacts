#[macro_use]
mod macros;
mod prim;
mod dict;
mod set;

#[cfg(test)]
mod tests;

pub(crate) use self::prim::{Merge, Split};
#[cfg(test)]
pub(crate) use self::set::{Arr64, Repr, Run16, Seq16};

pub(crate) const SEQ_MAX_LEN: usize = 4096;
pub(crate) const ARR_MAX_LEN: usize = 1024;
pub(crate) const U64_BITSIZE: usize = 64;
pub(crate) static TRUE: &bool = &true;
pub(crate) static FALSE: &bool = &false;

pub use self::dict::{PopCount, Rank, Select0, Select1};
pub use self::set::{Entries, Entry, Pair, Set};
pub use self::set::{And, AndNot, Or, Xor};
pub use self::set::{and, and_not, or, xor};

mod sealed {
    pub trait Op {}
    pub struct And;
    pub struct Or;
    pub struct AndNot;
    pub struct Xor;
    impl Op for And {}
    impl Op for Or {}
    impl Op for AndNot {}
    impl Op for Xor {}
}

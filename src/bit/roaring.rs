mod bin;
mod iter;
mod run;

use std::{borrow::Cow, ops::RangeInclusive};

use crate::bit::{self, ops::*};

type MapEncode = bit::Block<[u64; 1024]>;

/// Simply encoded bits block.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Block(pub(crate) Encode);

#[derive(Clone, Debug)]
pub(crate) enum Encode {
    Map(MapEncode),
    Bin(BinEncode),
    Run(RunEncode),
}

impl Default for Encode {
    fn default() -> Self {
        Encode::Bin(BinEncode::new())
    }
}

impl PartialEq for Encode {
    fn eq(&self, that: &Encode) -> bool {
        match (self, that) {
            (Encode::Map(b1), Encode::Map(b2)) => b1 == b2,
            (Encode::Bin(b1), Encode::Bin(b2)) => b1 == b2,
            (Encode::Run(b1), Encode::Run(b2)) => b1 == b2,
            _ => false,
        }
    }
}
impl Eq for Encode {}

/// Sorted vector of bits.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BinEncode(Vec<u16>);

/// Run length encoded.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RunEncode(Vec<RangeInclusive<u16>>);

// impl From<Block> for bit::Block {
//     fn from(block: Block) -> Self {
//         Array(block.0.into_boxed_slice())
//     }
// }

// impl<T: Into<Encode>> From<T> for Block {
//     fn from(b: T) -> Self {
//         Block(b.into())
//     }
// }

impl From<MapEncode> for Encode {
    fn from(data: MapEncode) -> Self {
        Encode::Map(data)
    }
}
impl From<BinEncode> for Encode {
    fn from(data: BinEncode) -> Self {
        Encode::Bin(data)
    }
}
impl From<RunEncode> for Encode {
    fn from(data: RunEncode) -> Self {
        Encode::Run(data)
    }
}

fn bin_to_map(bin: &BinEncode) -> MapEncode {
    let mut map = MapEncode::splat(0);
    for r in bin.runs() {
        let i = *r.start() as u64;
        let j = *r.end() as u64;
        map.set1(i..=j);
    }
    map
}

fn run_to_map(run: &RunEncode) -> MapEncode {
    let mut map = MapEncode::splat(0);
    for r in run.0.iter() {
        let i = *r.start() as u64;
        let j = *r.end() as u64;
        map.set1(i..=j);
    }
    map
}

impl Encode {
    const BIN_MAX_LEN: usize = (MapEncode::BITS / u16::BITS) as usize;

    fn into_block(self) -> MapEncode {
        match self {
            Encode::Map(map) => map,
            Encode::Bin(bin) => bin_to_map(&bin),
            Encode::Run(run) => run_to_map(&run),
        }
    }

    fn as_boxed_slice(&mut self) {
        match self {
            Encode::Map(_) => {}
            Encode::Bin(ref bin) => {
                *self = Encode::Map(bin_to_map(bin));
            }
            Encode::Run(ref run) => {
                *self = Encode::Map(run_to_map(run));
            }
        }
    }
}

macro_rules! delegate {
    ( $this:ident, $method:ident $(, $args:expr )* ) => {
        {
            match $this {
                Block(Encode::Map(data)) => data.$method( $( $args ),* ),
                Block(Encode::Bin(data)) => data.$method( $( $args ),* ),
                Block(Encode::Run(data)) => data.$method( $( $args ),* ),
            }
        }
    };
}

impl FiniteBits for Block {
    const BITS: u64 = MapEncode::BITS;
    fn empty() -> Self {
        Self::default()
    }
}

impl Count for Block {
    #[inline]
    fn bits(&self) -> u64 {
        Self::BITS
    }
    #[inline]
    fn count1(&self) -> u64 {
        delegate!(self, count1)
    }
}

impl Access for Block {
    fn access(&self, i: u64) -> bool {
        delegate!(self, access, i)
    }
}

impl Rank for Block {
    #[inline]
    fn rank1(&self, i: u64) -> u64 {
        delegate!(self, rank1, i)
    }
}

impl Select1 for Block {
    #[inline]
    fn select1(&self, n: u64) -> Option<u64> {
        delegate!(self, select1, n)
    }
}
impl Select0 for Block {
    #[inline]
    fn select0(&self, n: u64) -> Option<u64> {
        delegate!(self, select0, n)
    }
}

impl Assign<u64> for Block {
    type Output = ();
    fn set1(&mut self, i: u64) -> Self::Output {
        assert!(i < Self::BITS);
        match self {
            Block(Encode::Map(map)) => map.set1(i),
            Block(Encode::Bin(bin)) => {
                if !bin.access(i) {
                    bin.set1(i);
                    if bin.len() >= Encode::BIN_MAX_LEN {
                        *self = Block(Encode::Map(bin_to_map(&*bin)));
                    }
                }
            }
            Block(Encode::Run(run)) => run.set1(i),
        }
    }
    fn set0(&mut self, i: u64) -> Self::Output {
        delegate!(self, set0, i)
    }
}

impl Assign<std::ops::Range<u64>> for Block {
    type Output = ();
    fn set1(&mut self, i: std::ops::Range<u64>) {
        match self {
            Block(Encode::Map(map)) => {
                map.set1(i);
            }
            Block(Encode::Bin(bin)) => bin.set1(i),
            Block(Encode::Run(run)) => run.set1(i),
        }
    }
    fn set0(&mut self, i: std::ops::Range<u64>) {
        match self {
            Block(Encode::Map(map)) => {
                map.set0(i);
            }
            Block(Encode::Bin(bin)) => bin.set0(i),
            Block(Encode::Run(run)) => run.set0(i),
        }
    }
}

impl std::ops::BitAndAssign<&'_ Block> for Block {
    fn bitand_assign(&mut self, block: &Block) {
        self.0.bitand_assign(&block.0);
    }
}
impl std::ops::BitOrAssign<&'_ Block> for Block {
    fn bitor_assign(&mut self, block: &Block) {
        self.0.bitor_assign(&block.0);
    }
}
impl std::ops::BitXorAssign<&'_ Block> for Block {
    fn bitxor_assign(&mut self, block: &Block) {
        self.0.bitxor_assign(&block.0);
    }
}

impl std::ops::BitAndAssign<&'_ Encode> for Encode {
    fn bitand_assign(&mut self, encode: &Encode) {
        match (self, encode) {
            (Encode::Map(map1), Encode::Map(map2)) => map1.bitand_assign(map2),
            (Encode::Run(run1), Encode::Run(run2)) => run1.bitand_assign(run2),

            (Encode::Bin(bin1), Encode::Bin(bin2)) => bin1.bitand_assign(bin2),

            // FIXME: can be more efficient
            (Encode::Map(map), Encode::Bin(bin)) => map.bitand_assign(&bin_to_map(bin)),
            (Encode::Map(map), Encode::Run(run)) => map.bitand_assign(&run_to_map(run)),

            (Encode::Bin(bin), Encode::Map(map)) => bin.0.retain(|&x| map.access(u64::from(x))),

            (this @ Encode::Run(_), that @ Encode::Map(_)) => {
                this.as_boxed_slice();
                this.bitand_assign(that);
            }

            // FIXME: use Members
            (this @ Encode::Bin(_), that @ Encode::Run(_)) => {
                this.as_boxed_slice();
                this.bitand_assign(that);
            }
            (this @ Encode::Run(_), that @ Encode::Bin(_)) => {
                this.as_boxed_slice();
                this.bitand_assign(that);
            }
        }
    }
}

impl std::ops::BitOrAssign<&'_ Encode> for Encode {
    fn bitor_assign(&mut self, encode: &Encode) {
        match (self, encode) {
            (Encode::Map(map1), Encode::Map(map2)) => map1.bitor_assign(map2),
            (Encode::Run(run1), Encode::Run(run2)) => run1.bitor_assign(run2),

            // (Encode::Bin(bin1), Encode::Bin(bin2)) => bin1.bitor_assign(bin2),
            (Encode::Map(map), Encode::Bin(bin)) => {
                for &x in bin {
                    map.set1(u64::from(x));
                }
            }

            (Encode::Map(map), Encode::Run(run)) => {
                for r in run.0.iter() {
                    let i = u64::from(*r.start());;
                    let len = u64::from(*r.end()) - i + 1;
                    map.set1(i..len);
                }
            }

            (this @ Encode::Bin(_), that @ Encode::Map(_)) => {
                this.as_boxed_slice();
                this.bitor_assign(that);
            }

            (this @ Encode::Run(_), that @ Encode::Map(_)) => {
                this.as_boxed_slice();
                this.bitor_assign(that);
            }

            (this @ Encode::Bin(_), that @ Encode::Bin(_)) => {
                this.as_boxed_slice();
                this.bitor_assign(that);
            }

            // FIXME: use Members
            (this @ Encode::Bin(_), that @ Encode::Run(_)) => {
                this.as_boxed_slice();
                this.bitor_assign(that);
            }
            (this @ Encode::Run(_), that @ Encode::Bin(_)) => {
                this.as_boxed_slice();
                this.bitor_assign(that);
            }
        }
    }
}

impl std::ops::BitXorAssign<&'_ Encode> for Encode {
    fn bitxor_assign(&mut self, encode: &Encode) {
        match (self, encode) {
            (Encode::Map(map1), Encode::Map(map2)) => map1.bitxor_assign(map2),
            (Encode::Run(run1), Encode::Run(run2)) => run1.bitxor_assign(run2),

            // (Encode::Bin(bin1), Encode::Bin(bin2)) => bin1.bitxor_assign(bin2),
            (Encode::Map(map), Encode::Bin(bin)) => {
                for &x in bin {
                    let i = u64::from(x);
                    if map.access(i) {
                        map.set0(i);
                    } else {
                        map.set1(i);
                    }
                }
            }

            (Encode::Map(map), Encode::Run(run)) => map.bitxor_assign(&run_to_map(run)),

            (this @ Encode::Bin(_), that @ Encode::Map(_)) => {
                this.as_boxed_slice();
                this.bitxor_assign(that);
            }

            (this @ Encode::Run(_), that @ Encode::Map(_)) => {
                this.as_boxed_slice();
                this.bitxor_assign(that);
            }

            (this @ Encode::Bin(_), that @ Encode::Bin(_)) => {
                this.as_boxed_slice();
                this.bitxor_assign(that);
            }

            // FIXME: use Members
            (this @ Encode::Bin(_), that @ Encode::Run(_)) => {
                this.as_boxed_slice();
                this.bitxor_assign(that);
            }
            (this @ Encode::Run(_), that @ Encode::Bin(_)) => {
                this.as_boxed_slice();
                this.bitxor_assign(that);
            }
        }
    }
}

impl std::ops::Not for Encode {
    type Output = Encode;
    fn not(self) -> Self::Output {
        match self {
            Encode::Map(map) => Encode::Map(!map),
            Encode::Bin(bin) => Encode::Map(!bin),
            Encode::Run(run) => Encode::Run(!run),
        }
    }
}
impl std::ops::Not for &'_ Encode {
    type Output = Encode;
    fn not(self) -> Self::Output {
        match self {
            Encode::Map(ref map) => Encode::Map(!map),
            Encode::Bin(ref bin) => Encode::Map(!bin),
            Encode::Run(ref run) => Encode::Run(!run),
        }
    }
}

pub struct RoaringBlocks<'a> {
    iter: std::slice::Iter<'a, Block>,
}

pub struct RoaringEntries<'a, K: bit::Uint> {
    iter: std::slice::Iter<'a, bit::Entry<K, Block>>,
}

impl<'a> IntoIterator for &'a bit::Map<Block> {
    type Item = Cow<'a, Block>;
    type IntoIter = RoaringBlocks<'a>;
    fn into_iter(self) -> Self::IntoIter {
        let iter = self.data.iter();
        RoaringBlocks { iter }
    }
}

impl<'a, K> IntoIterator for &'a bit::KeyMap<K, Block>
where
    K: bit::Uint,
{
    type Item = bit::Entry<K, Cow<'a, Block>>;
    type IntoIter = RoaringEntries<'a, K>;
    fn into_iter(self) -> Self::IntoIter {
        let iter = self.data.iter();
        RoaringEntries { iter }
    }
}

impl<'a> Iterator for RoaringBlocks<'a> {
    type Item = Cow<'a, Block>;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(block) = self.iter.next() {
            if block.count1() > 0 {
                return Some(Cow::Borrowed(block));
            }
        }
        None
    }
}

impl<'a, K> Iterator for RoaringEntries<'a, K>
where
    K: bit::Uint,
{
    type Item = bit::Entry<K, Cow<'a, Block>>;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(page) = self.iter.next() {
            if page.value.count1() > 0 {
                let index = page.index;
                let value = Cow::Borrowed(&page.value);
                return Some(bit::Entry::new(index, value));
            }
        }
        None
    }
}

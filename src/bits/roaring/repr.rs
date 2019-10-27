use std::{fmt, io};

// use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt, LE};

// use crate::{
//     bits::{mask::*, FixedBits},
//     num::*,
//     ops::*,
// };

use crate::{
    bits::{Word, Words},
    ops::*,
};

use super::{Block, Loc1, Page, Repr, Run, Runs};

impl fmt::Debug for Repr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Repr::Loc1(this) => this.fmt(f),
            Repr::Page(this) => this.fmt(f),
            Repr::Runs(this) => this.fmt(f),
        }
    }
}

impl Default for Repr {
    fn default() -> Self {
        Repr::Loc1(Loc1::default())
    }
}

impl PartialEq for Repr {
    fn eq(&self, that: &Repr) -> bool {
        match (self, that) {
            (Repr::Loc1(this), Repr::Loc1(that)) => this.eq(that),
            (Repr::Runs(this), Repr::Runs(that)) => this.eq(that),
            (Repr::Page(this), Repr::Page(that)) => this.as_ref_words() == that.as_ref_words(),
            _ => false,
        }
    }
}
impl Eq for Repr {}

pub(crate) const LOC1_MAX: usize = Block::BITS / u16::SIZE;
pub(crate) const PAGE_LEN: usize = Block::BITS / u64::SIZE;

impl Repr {
    fn fit_to_bits(&mut self) {
        if let Repr::Loc1(loc1) = self {
            if loc1.data.len() > LOC1_MAX {
                *self = Repr::Page(Page::from(&*loc1));
            }
        }
    }
}

macro_rules! delegate {
    ( $this:ident, $method:ident $(, $args:expr )* ) => {
        {
            match $this {
                Repr::Loc1(data) => data.$method( $( $args ),* ),
                Repr::Page(data) => data.$method( $( $args ),* ),
                Repr::Runs(data) => data.$method( $( $args ),* ),
            }
        }
    };
}

// impl<'a> From<Cow<'a, crate::Bits<u64>>> for Block {
//     fn from(data: Cow<'a, crate::Bits<u64>>) -> Block {
//         unimplemented!()
//     }
// }

impl FixedBits for Block {
    const SIZE: usize = Block::BITS;
    fn none() -> Self {
        Self::default()
    }
}

impl FixedBits for Repr {
    const SIZE: usize = Block::BITS;
    fn none() -> Self {
        Self::default()
    }
}

impl Bits for Block {
    #[inline]
    fn size(&self) -> usize {
        Block::BITS
    }
    #[inline]
    fn count1(&self) -> usize {
        self.0.count1()
    }
    #[inline]
    fn count0(&self) -> usize {
        self.0.count0()
    }
    #[inline]
    fn all(&self) -> bool {
        self.0.all()
    }
    #[inline]
    fn any(&self) -> bool {
        self.0.any()
    }

    #[inline]
    fn bit(&self, i: usize) -> bool {
        self.0.bit(i)
    }

    #[inline]
    fn getn<W: Word>(&self, i: usize, n: usize) -> W {
        self.0.getn(i, n)
    }
}

impl Bits for Repr {
    #[inline]
    fn size(&self) -> usize {
        Self::SIZE
    }

    #[inline]
    fn count1(&self) -> usize {
        delegate!(self, count1)
    }
    #[inline]
    fn count0(&self) -> usize {
        delegate!(self, count0)
    }

    #[inline]
    fn all(&self) -> bool {
        delegate!(self, all)
    }
    #[inline]
    fn any(&self) -> bool {
        delegate!(self, any)
    }

    #[inline]
    fn bit(&self, i: usize) -> bool {
        delegate!(self, bit, i)
    }

    #[inline]
    fn getn<W: Word>(&self, i: usize, n: usize) -> W {
        delegate!(self, getn, i, n)
    }
}

impl BitsMut for Block {
    #[inline]
    fn put1(&mut self, i: usize) -> &mut Self {
        self.0.put1(i);
        self
    }
    #[inline]
    fn put0(&mut self, i: usize) -> &mut Self {
        self.0.put0(i);
        self
    }
    #[inline]
    fn flip(&mut self, i: usize) -> &mut Self {
        self.0.flip(i);
        self
    }

    #[inline]
    fn putn<W: Word>(&mut self, i: usize, n: usize, w: W) {
        self.0.putn(i, n, w)
    }
}

impl BitsMut for Repr {
    fn put1(&mut self, i: usize) -> &mut Self {
        match self {
            Repr::Loc1(data) => {
                let loc1 = data.put1(i);
                if loc1.data.len() > LOC1_MAX {
                    *self = Repr::Page(Page::from(&*loc1));
                }
            }

            Repr::Page(data) => {
                data.put1(i);
            }
            Repr::Runs(data) => {
                data.put1(i);
            }
        };
        self
    }

    fn put0(&mut self, i: usize) -> &mut Self {
        match self {
            Repr::Loc1(data) => {
                data.put0(i);
            }
            Repr::Page(data) => {
                data.put0(i);
            }
            Repr::Runs(data) => {
                data.put0(i);
            }
        };
        self
    }

    fn putn<W: Word>(&mut self, i: usize, n: usize, w: W) {
        delegate!(self, putn, i, n, w);
        self.fit_to_bits();
    }
}

impl BitRank for Block {
    #[inline]
    fn rank1(&self, i: usize, j: usize) -> usize {
        self.0.rank1(i, j)
    }
    #[inline]
    fn rank0(&self, i: usize, j: usize) -> usize {
        self.0.rank0(i, j)
    }
}

impl BitSelect for Block {
    #[inline]
    fn select1(&self, n: usize) -> Option<usize> {
        self.0.select1(n)
    }
    #[inline]
    fn select0(&self, n: usize) -> Option<usize> {
        self.0.select0(n)
    }
}

impl BitRank for Repr {
    #[inline]
    fn rank1(&self, i: usize, j: usize) -> usize {
        delegate!(self, rank1, i, j)
    }
    #[inline]
    fn rank0(&self, i: usize, j: usize) -> usize {
        delegate!(self, rank0, i, j)
    }
}

impl BitSelect for Repr {
    #[inline]
    fn select1(&self, n: usize) -> Option<usize> {
        delegate!(self, select1, n)
    }
    #[inline]
    fn select0(&self, n: usize) -> Option<usize> {
        delegate!(self, select0, n)
    }
}

// impl Repr {
//     pub(crate) fn is_runs(&self) -> bool {
//         if let Repr::Runs(_) = self {
//             true
//         } else {
//             false
//         }
//     }

//     pub(crate) fn optimize(&mut self) {
//         match self {
//             Repr::Loc1(heap) => {
//                 let mem_heap = size_of::U16 * heap.0.len();
//                 let mem_bits = size_of::U32 + size_of::U64 * REPR_BITS_LEN;
//                 let mem_runs = size_of::RUN * heap.runs().count();

//                 if mem_runs < std::cmp::min(mem_bits, mem_heap) {
//                     *self = Repr::Runs(Runs::from(&*heap));
//                 } else if heap.0.len() > REPR_POS1_LEN {
//                     *self = Repr::Page(Bits::from(&*heap))
//                 }
//             }

//             Repr::Page(bits) => {
//                 let pop = bits.count1() as usize;
//                 let mem_heap = size_of::U16 * pop;
//                 let mem_bits = size_of::U32 + size_of::U64 * REPR_BITS_LEN;
//                 let mem_runs = size_of::RUN * bits.runs();

//                 if mem_runs < std::cmp::min(mem_bits, mem_heap) {
//                     *self = Repr::Runs(Runs::from(&*bits));
//                 } else if pop <= REPR_POS1_LEN {
//                     *self = Repr::Loc1(Pos1::from(&*bits));
//                 }
//             }

//             Repr::Runs(runs) => {
//                 let pop = runs.count1() as usize;
//                 let mem_heap = size_of::U16 * pop;
//                 let mem_bits = size_of::U32 + size_of::U64 * REPR_BITS_LEN;
//                 let mem_runs = size_of::RUN * runs.0.len();

//                 if mem_runs > std::cmp::min(mem_bits, mem_heap) {
//                     if pop <= REPR_POS1_LEN {
//                         *self = Repr::Loc1(Pos1::from(&*runs))
//                     } else {
//                         *self = Repr::Page(Bits::from(&*runs))
//                     }
//                 }
//             }
//         };
//     }

//     pub(crate) fn read_heap_from<R: io::Read>(mut r: R, pop: usize) -> io::Result<Self> {
//         let mut vec = vec![0; pop];
//         r.read_u16_into::<LE>(&mut vec)
//             .map(|()| Repr::Loc1(posn::Pos1(vec)))
//     }

//     pub(crate) fn read_bits_from<R: io::Read>(mut r: R, _pop: usize) -> io::Result<Self> {
//         let mut vec = vec![0; REPR_BITS_LEN];
//         r.read_usize_into::<LE>(&mut vec)
//             .map(|()| Repr::Page({ vec.into_boxed_slice().into() }))
//     }

//     pub(crate) fn read_runs_from<R: io::Read>(mut r: R, pop: usize) -> io::Result<Self> {
//         let vec_len = r.read_u16::<LE>()? as usize;
//         let mut ones = 0;
//         let mut runs = Vec::with_capacity(vec_len);

//         for _ in 0..vec_len {
//             let s = r.read_u16::<LE>()?;
//             let o = r.read_u16::<LE>()?;
//             ones += u32::from(o) + 1;
//             runs.push(Bounds(s, s + o));
//         }
//         assert_eq!(ones as usize, pop);
//         Ok(Repr::Runs(runs::Runs(runs)))
//     }

//     pub(crate) fn heap_from_bytes(bytes: &[u8], pop: usize) -> Self {
//         let mut vec = vec![0; pop];
//         LE::read_u16_into(&bytes[..pop * 2], &mut vec);
//         Repr::Loc1(posn::Pos1(vec))
//     }

//     pub(crate) fn bits_from_bytes(bytes: &[u8], _pop: usize) -> Self {
//         let mut vec = FixedBits::<[usize; REPR_BITS_LEN]>::splat(0);
//         LE::read_usize_into(&bytes[..REPR_BITS_LEN * 8], &mut vec.0);
//         Repr::Page(vec)
//     }

//     pub(crate) fn runs_from_bytes(bytes: &[u8], pop: usize) -> Self {
//         let vec_len = LE::read_u16(&bytes[..2]) as usize;
//         let mut ones = 0;
//         let mut runs = Vec::with_capacity(vec_len);

//         let mut p = 2;
//         for _ in 0..vec_len {
//             let s = LE::read_u16(&bytes[p..p + 2]);
//             let o = LE::read_u16(&bytes[p + 2..p + 4]);
//             p += 4;
//             ones += u32::from(o) + 1;
//             runs.push(Bounds(s, s + o));
//         }
//         assert_eq!(ones as usize, pop);
//         Repr::Runs(runs::Runs(runs))
//     }

//     pub(crate) fn bytes_len(&self) -> usize {
//         match self {
//             Repr::Loc1(r) => size_of::U16 * r.0.len(),
//             Repr::Page(_) => size_of::U64 * REPR_BITS_LEN,
//             Repr::Runs(r) => size_of::U16 + size_of::U16 * 2 * r.0.len(),
//         }
//     }

//     pub(crate) fn write_to<W: io::Write>(&self, mut w: W) -> io::Result<()> {
//         let ones = self.count1();
//         match self {
//             Repr::Loc1(heap) => {
//                 assert!(ones <= REPR_POS1_LEN as usize);
//                 for &b in heap.as_ref() {
//                     w.write_u16::<LE>(b)?;
//                 }
//             }
//             Repr::Page(bits) => {
//                 assert!(ones > REPR_POS1_LEN as usize);
//                 for &b in bits.iter() {
//                     w.write_usize::<LE>(b)?;
//                 }
//             }
//             Repr::Runs(runs) => {
//                 w.write_u16::<LE>(try_cast(runs.0.len()))?;
//                 for Bounds(i, j) in runs.as_ref() {
//                     // let i = *run.start();
//                     // let j = *run.end();
//                     w.write_u16::<LE>(*i)?;
//                     w.write_u16::<LE>(*j - *i)?;
//                 }
//             }
//         };
//         Ok(())
//     }

//     fn bits_mut_then(&mut self, mut func: impl FnMut(&mut Bits)) {
//         match self {
//             Repr::Page(ref mut bits) => {
//                 func(bits);
//             }
//             Repr::Loc1(heap) => {
//                 let mut bits = Bits::from(&*heap);
//                 func(&mut bits);
//                 *self = Repr::Page(bits);
//             }
//             Repr::Runs(runs) => {
//                 let mut bits = Bits::from(&*runs);
//                 func(&mut bits);
//                 *self = Repr::Page(bits);
//             }
//         }
//     }
// }

// impl Intersection<Pos1> for Repr {
//     fn intersection(&mut self, heap: &Pos1) {
//         match self {
//             Repr::Loc1(this) => this.intersection(heap),
//             Repr::Page(this) => {
//                 let mut new_heap = heap.clone();
//                 new_heap.intersection(&*this);
//                 *self = Repr::Loc1(new_heap);
//             }
//             this @ Repr::Runs(_) => {
//                 let bits = Bits::from(&*heap);
//                 this.bits_mut_then(|new_bits| new_bits.intersection(&bits))
//             }
//         }
//     }
// }

// impl Intersection<Bits> for Repr {
//     #[inline]
//     fn intersection(&mut self, bits: &Bits) {
//         match self {
//             Repr::Loc1(this) => this.intersection(&*bits),
//             Repr::Page(this) => this.intersection(bits),
//             this @ Repr::Runs(_) => this.bits_mut_then(|new_bits| new_bits.intersection(bits)),
//         }
//     }
// }

// impl Intersection<Runs> for Repr {
//     #[inline]
//     fn intersection(&mut self, runs: &Runs) {
//         let bits = Bits::from(&*runs);
//         self.bits_mut_then(|new_bits| new_bits.intersection(&bits));
//     }
// }

// impl Intersection<Self> for Repr {
//     #[inline]
//     fn intersection(&mut self, repr: &Repr) {
//         match repr {
//             Repr::Loc1(that) => self.intersection(that),
//             Repr::Page(that) => self.intersection(that),
//             Repr::Runs(that) => self.intersection(that),
//         }
//     }
// }

// impl Union<Pos1> for Repr {
//     #[inline]
//     fn union(&mut self, heap: &Pos1) {
//         match self {
//             this @ Repr::Loc1(_) | this @ Repr::Runs(_) => {
//                 this.bits_mut_then(|new_bits| new_bits.union(heap))
//             }
//             Repr::Page(this) => this.union(heap),
//         }
//     }
// }

// impl Union<Bits> for Repr {
//     #[inline]
//     fn union(&mut self, bits: &Bits) {
//         self.bits_mut_then(|this| this.union(&**bits))
//     }
// }

// impl Union<Runs> for Repr {
//     #[inline]
//     fn union(&mut self, runs: &Runs) {
//         match self {
//             this @ Repr::Loc1(_) => this.bits_mut_then(|new_bits| new_bits.union(runs)),
//             Repr::Page(this) => this.union(runs),
//             Repr::Runs(this) => this.union(runs),
//             // Repr::Runs(this) => this.bits_mut_then(|new_bits| {
//             //     new_bits.union(runs);
//             // }),
//         }
//     }
// }

// impl Union<Self> for Repr {
//     #[inline]
//     fn union(&mut self, repr: &Self) {
//         match repr {
//             Repr::Loc1(that) => self.union(that),
//             Repr::Page(that) => self.union(that),
//             Repr::Runs(that) => self.union(that),
//         }
//     }
// }

// impl Difference<Pos1> for Repr {
//     #[inline]
//     fn difference(&mut self, heap: &Pos1) {
//         match self {
//             Repr::Loc1(this) => this.difference(heap),
//             Repr::Page(this) => this.difference(heap),
//             this @ Repr::Runs(_) => this.bits_mut_then(|new_bits| new_bits.difference(heap)),
//         }
//     }
// }

// impl Difference<Bits> for Repr {
//     #[inline]
//     fn difference(&mut self, bits: &Bits) {
//         match self {
//             Repr::Loc1(this) => this.difference(&*bits),
//             Repr::Page(this) => this.difference(&**bits),
//             this @ Repr::Runs(_) => this.bits_mut_then(|new_bits| new_bits.difference(&**bits)),
//         }
//     }
// }

// impl Difference<Runs> for Repr {
//     #[inline]
//     fn difference(&mut self, runs: &Runs) {
//         match self {
//             this @ Repr::Loc1(_) => this.bits_mut_then(|new_bits| new_bits.difference(runs)),
//             Repr::Page(this) => this.difference(runs),
//             Repr::Runs(this) => this.difference(runs),
//         }
//     }
// }

// impl Difference<Self> for Repr {
//     #[inline]
//     fn difference(&mut self, repr: &Self) {
//         match repr {
//             Repr::Loc1(that) => self.difference(that),
//             Repr::Page(that) => self.difference(that),
//             Repr::Runs(that) => self.difference(that),
//         }
//     }
// }

// impl SymmetricDifference<Pos1> for Repr {
//     fn symmetric_difference(&mut self, heap: &Pos1) {
//         match self {
//             Repr::Loc1(this) => this.symmetric_difference(heap),
//             Repr::Page(this) => this.symmetric_difference(heap),
//             this @ Repr::Runs(_) => {
//                 this.bits_mut_then(|new_bits| new_bits.symmetric_difference(heap))
//             }
//         }
//     }
// }

// impl SymmetricDifference<Bits> for Repr {
//     fn symmetric_difference(&mut self, bits: &Bits) {
//         self.bits_mut_then(|this| this.symmetric_difference(&**bits))
//     }
// }

// impl SymmetricDifference<Runs> for Repr {
//     fn symmetric_difference(&mut self, runs: &Runs) {
//         match self {
//             this @ Repr::Loc1(_) => {
//                 this.bits_mut_then(|new_bits| new_bits.symmetric_difference(runs))
//             }
//             Repr::Page(this) => this.symmetric_difference(runs),
//             Repr::Runs(this) => this.symmetric_difference(runs),
//         }
//     }
// }

// impl SymmetricDifference<Self> for Repr {
//     #[inline]
//     fn symmetric_difference(&mut self, repr: &Self) {
//         match repr {
//             Repr::Loc1(that) => self.symmetric_difference(that),
//             Repr::Page(that) => self.symmetric_difference(that),
//             Repr::Runs(that) => self.symmetric_difference(that),
//         }
//     }
// }

// impl Intersection<Self> for Block {
//     #[inline]
//     fn intersection(&mut self, that: &Block) {
//         self.0.intersection(&that.0);
//     }
// }

// impl Union<Self> for Block {
//     #[inline]
//     fn union(&mut self, that: &Block) {
//         self.0.union(&that.0);
//     }
// }

// impl Difference<Self> for Block {
//     #[inline]
//     fn difference(&mut self, that: &Block) {
//         self.0.difference(&that.0);
//     }
// }

// impl SymmetricDifference<Self> for Block {
//     #[inline]
//     fn symmetric_difference(&mut self, that: &Block) {
//         self.0.symmetric_difference(&that.0);
//     }
// }

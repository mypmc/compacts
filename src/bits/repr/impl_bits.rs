use std::cmp;
use bits::{self, pair};
use super::{ArrBlock, RunBlock, SeqBlock};

impl<'a> bits::BitAndAssign<&'a SeqBlock> for SeqBlock {
    fn bitand_assign(&mut self, seq: &'a SeqBlock) {
        *self = {
            let data = pair::and(&*self, seq).filter_map(|tup| match tup {
                (Some(l), Some(r)) if l == r => Some(l),
                _ => None,
            });
            let min = cmp::min(self.vector.len(), seq.vector.len());
            let mut seq = SeqBlock::with_capacity(min);
            for bit in data {
                seq.insert(bit);
            }
            seq
        };
    }
}

impl<'a> bits::BitOrAssign<&'a SeqBlock> for SeqBlock {
    fn bitor_assign(&mut self, seq: &'a SeqBlock) {
        for &bit in &seq.vector {
            self.insert(bit);
        }
    }
}

impl<'a> bits::BitAndNotAssign<&'a SeqBlock> for SeqBlock {
    fn bitandnot_assign(&mut self, seq: &'a SeqBlock) {
        *self = {
            let data = pair::and_not(&*self, seq).filter_map(|tup| match tup {
                (Some(l), None) => Some(l),
                _ => None,
            });
            let mut seq = SeqBlock::with_capacity(self.vector.len());
            for bit in data {
                seq.insert(bit);
            }
            seq
        };
    }
}

impl<'a> bits::BitXorAssign<&'a SeqBlock> for SeqBlock {
    fn bitxor_assign(&mut self, seq: &'a SeqBlock) {
        for &bit in &seq.vector {
            if self.insert(bit) {
                self.remove(bit);
            }
        }
    }
}

impl<'a> bits::BitAndAssign<&'a ArrBlock> for ArrBlock {
    fn bitand_assign(&mut self, arr: &'a ArrBlock) {
        assert_eq!(self.bitmap.len(), arr.bitmap.len());
        self.weight = {
            let mut new = 0;
            for (x, y) in self.bitmap.iter_mut().zip(arr.bitmap.iter()) {
                *x &= *y;
                new += x.count_ones();
            }
            new
        };
    }
}

impl<'a> bits::BitOrAssign<&'a ArrBlock> for ArrBlock {
    fn bitor_assign(&mut self, arr: &'a ArrBlock) {
        assert_eq!(self.bitmap.len(), arr.bitmap.len());
        self.weight = {
            let mut new = 0;
            for (x, y) in self.bitmap.iter_mut().zip(arr.bitmap.iter()) {
                *x |= *y;
                new += x.count_ones();
            }
            new
        };
    }
}

impl<'a> bits::BitAndNotAssign<&'a ArrBlock> for ArrBlock {
    fn bitandnot_assign(&mut self, arr: &'a ArrBlock) {
        assert_eq!(self.bitmap.len(), arr.bitmap.len());
        self.weight = {
            let mut new = 0;
            for (x, y) in self.bitmap.iter_mut().zip(arr.bitmap.iter()) {
                *x &= !*y;
                new += x.count_ones();
            }
            new
        };
    }
}

impl<'a> bits::BitXorAssign<&'a ArrBlock> for ArrBlock {
    fn bitxor_assign(&mut self, arr: &'a ArrBlock) {
        assert_eq!(self.bitmap.len(), arr.bitmap.len());
        self.weight = {
            let mut new = 0;
            for (x, y) in self.bitmap.iter_mut().zip(arr.bitmap.iter()) {
                *x ^= *y;
                new += x.count_ones();
            }
            new
        };
    }
}

impl<'a> bits::BitAndAssign<&'a RunBlock> for RunBlock {
    fn bitand_assign(&mut self, rle16: &'a RunBlock) {
        let new = self.and(rle16).collect();
        *self = new;
    }
}
impl<'a> bits::BitOrAssign<&'a RunBlock> for RunBlock {
    fn bitor_assign(&mut self, rle16: &'a RunBlock) {
        let new = self.or(rle16).collect();
        *self = new;
    }
}
impl<'a> bits::BitAndNotAssign<&'a RunBlock> for RunBlock {
    fn bitandnot_assign(&mut self, rle16: &'a RunBlock) {
        let new = self.and_not(rle16).collect();
        *self = new;
    }
}
impl<'a> bits::BitXorAssign<&'a RunBlock> for RunBlock {
    fn bitxor_assign(&mut self, rle16: &'a RunBlock) {
        let new = self.xor(rle16).collect();
        *self = new;
    }
}

// impl<'a> bits::BitOrAssign<&'a RunBlock> for RunBlock {
//     fn bitor_assign(&mut self, rle16: &'a RunBlock) {
//         *self = twofold_filter!(self, rle16, filter_or);
//     }
// }
// impl<'a> bits::BitAndNotAssign<&'a RunBlock> for RunBlock {
//     fn bitandnot_assign(&mut self, rle16: &'a RunBlock) {
//         *self = twofold_filter!(self, rle16, filter_and_not);
//     }
// }
// impl<'a> bits::BitXorAssign<&'a RunBlock> for RunBlock {
//     fn bitxor_assign(&mut self, rle16: &'a RunBlock) {
//         *self = twofold_filter!(self, rle16, filter_xor);
//     }
// }

use std::ops;
use super::{pair, Bits, Repr};

macro_rules! clone_symmetric_difference {
    ( $clone: ident, $source: expr, $target: expr ) => {
        let mut $clone = $source.clone();
        $clone.symmetric_difference_with($target);
    };
}

impl Repr {
    fn symmetric_difference_with(&mut self, other: &Repr) {
        match (self, other) {
            (this @ &mut Repr::Vec(..), that @ &Repr::Vec(..)) => {
                let repr = this.clone();
                let iter = pair!(symmetric_difference, repr, that);
                *this = iter.collect::<Repr>();
            }

            (this @ &mut Repr::Vec(..), that @ &Repr::Map(..)) => {
                clone_symmetric_difference!(clone, that, this);
                *this = clone;
            }

            (ref mut this @ &mut Repr::Map(..), &Repr::Vec(_, ref vec)) => {
                for &bit in vec.iter() {
                    if this.contains(bit) {
                        this.remove(bit);
                    } else {
                        this.insert(bit);
                    }
                }
            }
            (&mut Repr::Map(ref mut ones, ref mut map1), &Repr::Map(_, ref map2)) => {
                *ones = 0;
                for (x, y) in map1.iter_mut().zip(map2.iter()) {
                    let p = *x ^ *y;
                    *ones += p.ones() as usize;
                    *x = p;
                }
            }
        }
    }
}

impl<'a, 'b> ops::BitXor<&'b Repr> for &'a Repr {
    type Output = Repr;
    fn bitxor(self, other: &Repr) -> Self::Output {
        match (self, other) {
            (this @ &Repr::Vec(..), that @ &Repr::Vec(..)) => {
                let iter = pair!(symmetric_difference, this, that);
                iter.collect::<Repr>()
            }
            (this @ &Repr::Vec(..), that @ &Repr::Map(..)) => {
                clone_symmetric_difference!(clone, that, this);
                clone
            }
            (this, that) => {
                clone_symmetric_difference!(clone, this, that);
                clone
            }
        }
    }
}
impl<'a> ops::BitXorAssign<&'a Repr> for Repr {
    fn bitxor_assign(&mut self, other: &Repr) {
        self.symmetric_difference_with(other);
    }
}

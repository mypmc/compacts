use std::ops;
use super::{pair, Bits, Repr};

macro_rules! clone_union_with {
    ( $clone: ident, $source: expr, $target: expr ) => {
        let mut $clone = $source.clone();
        $clone.union_with($target);
    };
}

impl Repr {
    fn union_with(&mut self, other: &Repr) {
        match (self, other) {
            (this @ &mut Repr::Vec(..), that @ &Repr::Vec(..)) => {
                let repr = this.clone();
                let iter = pair!(union, repr, that);
                *this = iter.collect::<Repr>();
            }

            (mut this @ &mut Repr::Vec(..), that @ &Repr::Map(..)) => {
                clone_union_with!(clone, that, this);
                *this = clone;
            }

            (ref mut this @ &mut Repr::Map(..), &Repr::Vec(_, ref vec)) => {
                for &bit in vec {
                    this.insert(bit);
                }
            }

            (&mut Repr::Map(ref mut ones, ref mut map1), &Repr::Map(_, ref map2)) => {
                *ones = 0;
                for (x, y) in map1.iter_mut().zip(map2.iter()) {
                    let p = *x | *y;
                    *ones += p.ones();
                    *x = p;
                }
            }
        }
    }
}

impl<'a, 'b> ops::BitOr<&'b Repr> for &'a Repr {
    type Output = Repr;
    fn bitor(self, other: &Repr) -> Self::Output {
        match (self, other) {
            (this @ &Repr::Vec(..), that @ &Repr::Vec(..)) => {
                let iter = pair!(union, this, that);
                iter.collect::<Repr>()
            }
            (this @ &Repr::Vec(..), that @ &Repr::Map(..)) => {
                clone_union_with!(clone, that, this);
                clone
            }
            (this, that) => {
                clone_union_with!(clone, this, that);
                clone
            }
        }
    }
}
impl<'a> ops::BitOrAssign<&'a Repr> for Repr {
    fn bitor_assign(&mut self, other: &Repr) {
        self.union_with(other);
    }
}

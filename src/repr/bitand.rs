use std::ops;
use super::{pair, Bits, Repr};

macro_rules! clone_intersect_with {
    ( $clone: ident, $source: expr, $target: expr ) => {
        let mut $clone = $source.clone();
        $clone.intersect_with($target);
    };
}

impl Repr {
    fn intersect_with(&mut self, other: &Repr) {
        match (self, other) {
            (this @ &mut Repr::Vec(..), that @ &Repr::Vec(..)) => {
                let repr = this.clone();
                let iter = pair!(intersection, repr, that);
                *this = iter.collect::<Repr>();
            }

            (&mut Repr::Vec(ref mut ones, ref mut vec), that @ &Repr::Map(..)) => {
                *ones = 0;
                for i in 0..vec.len() {
                    if that.contains(vec[i]) {
                        vec[*ones] = vec[i];
                        *ones += 1;
                    }
                }
                vec.truncate(*ones);
            }

            (this @ &mut Repr::Map(..), that @ &Repr::Vec(..)) => {
                clone_intersect_with!(clone, that, this);
                *this = clone;
            }

            (&mut Repr::Map(ref mut ones, ref mut map1), &Repr::Map(_, ref map2)) => {
                *ones = 0;
                for (x, y) in map1.iter_mut().zip(map2.iter()) {
                    let p = *x & *y;
                    *ones += p.ones() as usize;
                    *x = p;
                }
            }
        }
    }
}

impl<'a, 'b> ops::BitAnd<&'b Repr> for &'a Repr {
    type Output = Repr;
    fn bitand(self, other: &Repr) -> Self::Output {
        match (self, other) {
            (this @ &Repr::Vec(..), that @ &Repr::Vec(..)) => {
                let iter = pair!(intersection, this, that);
                iter.collect::<Repr>()
            }
            (this @ &Repr::Map(..), that @ &Repr::Vec(..)) => {
                clone_intersect_with!(clone, that, this);
                clone
            }
            (this, that) => {
                clone_intersect_with!(clone, this, that);
                clone
            }
        }
    }
}
impl<'a> ops::BitAndAssign<&'a Repr> for Repr {
    fn bitand_assign(&mut self, other: &Repr) {
        self.intersect_with(other)
    }
}

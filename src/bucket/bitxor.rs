use std::ops;
use super::{pair, Bits, Count, Bucket};

macro_rules! clone_symmetric_difference {
    ( $clone: ident, $source: expr, $target: expr ) => {
        let mut $clone = $source.clone();
        $clone.symmetric_difference_with($target);
    };
}

impl Bucket {
    fn symmetric_difference_with(&mut self, other: &Bucket) {
        match (self, other) {
            (this @ &mut Bucket::Vec(..), that @ &Bucket::Vec(..)) => {
                let repr = this.clone();
                let iter = pair!(symmetric_difference, repr, that);
                *this = iter.collect::<Bucket>();
            }

            (this @ &mut Bucket::Vec(..), that @ &Bucket::Map(..)) => {
                clone_symmetric_difference!(clone, that, this);
                *this = clone;
            }

            (ref mut this @ &mut Bucket::Map(..), &Bucket::Vec(_, ref vec)) => {
                for &bit in vec.iter() {
                    if this.contains(bit) {
                        this.remove(bit);
                    } else {
                        this.insert(bit);
                    }
                }
            }
            (&mut Bucket::Map(ref mut pop, ref mut map1), &Bucket::Map(_, ref map2)) => {
                let mut ones = 0;
                for (x, y) in map1.iter_mut().zip(map2.iter()) {
                    let p = *x ^ *y;
                    ones += p.ones();
                    *x = p;
                }
                *pop = Count::<u16>::new(ones);
            }
        }
    }
}

impl<'a, 'b> ops::BitXor<&'b Bucket> for &'a Bucket {
    type Output = Bucket;
    fn bitxor(self, other: &Bucket) -> Self::Output {
        match (self, other) {
            (this @ &Bucket::Vec(..), that @ &Bucket::Vec(..)) => {
                let iter = pair!(symmetric_difference, this, that);
                iter.collect::<Bucket>()
            }
            (this @ &Bucket::Vec(..), that @ &Bucket::Map(..)) => {
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
impl<'a> ops::BitXorAssign<&'a Bucket> for Bucket {
    fn bitxor_assign(&mut self, other: &Bucket) {
        self.symmetric_difference_with(other);
    }
}

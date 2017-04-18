use std::ops;
use super::{pair, Bits, PopCount, Bucket};

macro_rules! clone_union_with {
    ( $clone: ident, $source: expr, $target: expr ) => {
        let mut $clone = $source.clone();
        $clone.union_with($target);
    };
}

impl Bucket {
    fn union_with(&mut self, other: &Bucket) {
        match (self, other) {
            (this @ &mut Bucket::Vec(..), that @ &Bucket::Vec(..)) => {
                let repr = this.clone();
                let iter = pair!(union, repr, that);
                *this = iter.collect::<Bucket>();
            }

            (mut this @ &mut Bucket::Vec(..), that @ &Bucket::Map(..)) => {
                clone_union_with!(clone, that, this);
                *this = clone;
            }

            (ref mut this @ &mut Bucket::Map(..), &Bucket::Vec(_, ref vec)) => {
                for &bit in vec {
                    this.insert(bit);
                }
            }

            (&mut Bucket::Map(ref mut pop, ref mut map1), &Bucket::Map(_, ref map2)) => {
                let mut ones = 0;
                for (x, y) in map1.iter_mut().zip(map2.iter()) {
                    let p = *x | *y;
                    ones += p.ones();
                    *x = p;
                }
                *pop = PopCount::<u16>::new(ones);
            }
        }
    }
}

impl<'a, 'b> ops::BitOr<&'b Bucket> for &'a Bucket {
    type Output = Bucket;
    fn bitor(self, other: &Bucket) -> Self::Output {
        match (self, other) {
            (this @ &Bucket::Vec(..), that @ &Bucket::Vec(..)) => {
                let iter = pair!(union, this, that);
                iter.collect::<Bucket>()
            }
            (this @ &Bucket::Vec(..), that @ &Bucket::Map(..)) => {
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
impl<'a> ops::BitOrAssign<&'a Bucket> for Bucket {
    fn bitor_assign(&mut self, other: &Bucket) {
        self.union_with(other);
    }
}

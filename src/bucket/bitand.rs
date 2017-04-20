use std::ops;

use {bits, PopCount, Bucket};
use bucket::pair;

macro_rules! clone_intersect_with {
    ( $clone: ident, $source: expr, $target: expr ) => {
        let mut $clone = $source.clone();
        $clone.intersect_with($target);
    };
}

impl Bucket {
    fn intersect_with(&mut self, other: &Bucket) {
        match (self, other) {
            (this @ &mut Bucket::Vec(..), that @ &Bucket::Vec(..)) => {
                let repr = this.clone();
                let iter = pair!(intersection, repr, that);
                *this = iter.collect::<Bucket>();
            }

            (&mut Bucket::Vec(ref mut pop, ref mut vec), that @ &Bucket::Map(..)) => {
                let mut ones = 0;
                for i in 0..vec.len() {
                    if that.contains(vec[i]) {
                        vec[ones] = vec[i];
                        ones += 1;
                    }
                }
                *pop = bits::Count::<u16>::new(ones as u64);
                vec.truncate(ones);
            }

            (this @ &mut Bucket::Map(..), that @ &Bucket::Vec(..)) => {
                clone_intersect_with!(clone, that, this);
                *this = clone;
            }

            (&mut Bucket::Map(ref mut pop, ref mut map1), &Bucket::Map(_, ref map2)) => {
                let mut ones = 0;
                for (x, y) in map1.iter_mut().zip(map2.iter()) {
                    let p = *x & *y;
                    ones += p.ones();
                    *x = p;
                }
                *pop = bits::Count::<u16>::new(ones);
            }
        }
    }
}

impl<'a, 'b> ops::BitAnd<&'b Bucket> for &'a Bucket {
    type Output = Bucket;
    fn bitand(self, other: &Bucket) -> Self::Output {
        match (self, other) {
            (this @ &Bucket::Vec(..), that @ &Bucket::Vec(..)) => {
                let iter = pair!(intersection, this, that);
                iter.collect::<Bucket>()
            }
            (this @ &Bucket::Map(..), that @ &Bucket::Vec(..)) => {
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
impl<'a> ops::BitAndAssign<&'a Bucket> for Bucket {
    fn bitand_assign(&mut self, other: &Bucket) {
        self.intersect_with(other)
    }
}

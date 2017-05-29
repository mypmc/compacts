use ops::*;

use super::Block;
use super::Block::*;

macro_rules! impl_op {
    ( $op:ident, $fn:ident, $fn_with:ident ) => {
        impl<'a, 'b> $op<&'b Block> for &'a Block {
            type Output = Block;
            fn $fn(self, that: &Block) -> Self::Output {
                match (self, that) {
                    (this @ &Vec16(..), that @ &Vec16(..)) => {
                        let pair = ::pairwise::$fn(this.iter(), that.iter());
                        pair.collect()
                    }
                    (this, that) => {
                        let mut cloned = this.clone();
                        cloned.$fn_with(that);
                        cloned
                    }
                }
            }
        }

    }
}

impl_op!(Intersection, intersection, intersection_with);
impl_op!(Union, union, union_with);
impl_op!(Difference, difference, difference_with);
impl_op!(SymmetricDifference,
         symmetric_difference,
         symmetric_difference_with);

impl<'a> IntersectionWith<&'a Block> for Block {
    fn intersection_with(&mut self, target: &Block) {
        match (self, target) {
            (&mut Vec64(ref mut b1), &Vec64(ref b2)) => {
                bucket_foreach!(b1 & b2);
            }

            (&mut Vec64(ref mut b1), &Vec16(ref b2)) => {
                let b3 = super::inner::Seq64::from(b2);
                bucket_foreach!(b1 & b3);
            }

            (&mut Vec16(ref mut b), that @ &Vec64(..)) => {
                let weight = {
                    let mut weight = 0;
                    for i in 0..b.vector.len() {
                        if that.contains(b.vector[i]) {
                            b.vector[weight] = b.vector[i];
                            weight += 1;
                        }
                    }
                    weight
                };
                b.vector.truncate(weight);
                b.weight = weight as u32;
            }

            (this, that) => {
                *this = {
                    let pair = ::pairwise::intersection(this.iter(), that.iter());
                    pair.collect()
                };
            }
        }
    }
}

impl<'a> UnionWith<&'a Block> for Block {
    fn union_with(&mut self, target: &Block) {
        if target.count_ones() == 0 {
            return;
        }
        match (self, target) {
            (&mut Vec64(ref mut b1), &Vec64(ref b2)) => {
                bucket_foreach!(b1 | b2);
            }

            (&mut Vec64(ref mut b1), &Vec16(ref b2)) => {
                for &bit in &b2.vector[..] {
                    b1.insert(bit);
                }
            }

            (this @ &mut Vec16(..), that @ &Vec64(..)) => {
                this.as_mapped();
                this.union_with(that)
            }

            (this, that) => {
                *this = {
                    let pair = ::pairwise::union(this.iter(), that.iter());
                    pair.collect()
                };
            }
        }
    }
}

impl<'a> DifferenceWith<&'a Block> for Block {
    fn difference_with(&mut self, target: &Block) {
        match (self, target) {
            (&mut Vec64(ref mut b1), &Vec64(ref b2)) => {
                bucket_foreach!(b1 - b2);
            }
            (&mut Vec64(ref mut b1), &Vec16(ref b2)) => {
                for &item in &b2.vector[..] {
                    b1.remove(item);
                }
            }

            (this @ &mut Vec16(..), that @ &Vec64(..)) => {
                this.as_mapped();
                this.difference_with(that);
            }

            (this, that) => {
                *this = {
                    let pair = ::pairwise::difference(this.iter(), that.iter());
                    pair.collect()
                };
            }
        }
    }
}

impl<'a> SymmetricDifferenceWith<&'a Block> for Block {
    fn symmetric_difference_with(&mut self, target: &Block) {
        match (self, target) {
            (&mut Vec64(ref mut b1), &Vec64(ref b2)) => {
                bucket_foreach!(b1 ^ b2);
            }

            (ref mut this @ &mut Vec64(..), &Vec16(ref b)) => {
                for &bit in &b.vector {
                    if this.contains(bit) {
                        this.remove(bit);
                    } else {
                        this.insert(bit);
                    }
                }
            }

            (this @ &mut Vec16(..), that @ &Vec64(..)) => {
                this.as_mapped();
                this.symmetric_difference_with(that)
            }

            (this, that) => {
                *this = {
                    let pair = ::pairwise::symmetric_difference(this.iter(), that.iter());
                    pair.collect()
                };
            }
        }
    }
}

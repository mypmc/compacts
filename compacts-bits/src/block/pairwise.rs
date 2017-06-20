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
                        ::pairwise::$fn(this.iter(), that.iter()).collect()
                    }
                    (&Rle16(ref this), &Rle16(ref that)) => Rle16(this.intersection(that)),

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
impl_op!(
    SymmetricDifference,
    symmetric_difference,
    symmetric_difference_with
);

impl<'a> IntersectionWith<&'a Block> for Block {
    fn intersection_with(&mut self, target: &Block) {
        match (self, target) {
            (&mut Vec16(ref mut b1), &Vec16(ref b2)) => b1.intersection_with(b2),
            (&mut Vec16(ref mut b1), &Vec64(ref b2)) => b1.intersection_with(b2),

            (&mut Vec64(ref mut b1), &Vec16(ref b2)) => b1.intersection_with(b2),
            (&mut Vec64(ref mut b1), &Vec64(ref b2)) => b1.intersection_with(b2),
            (&mut Vec64(ref mut b1), &Rle16(ref b2)) => b1.intersection_with(b2),

            (&mut Rle16(ref mut b1), &Rle16(ref b2)) => b1.intersection_with(b2),

            (this, that) => {
                this.as_vec64();
                this.intersection_with(that);
            }
        }
    }
}

macro_rules! impl_mutop {
    ( $op:ident, $fn_with:ident ) => {
        impl<'a> $op<&'a Block> for Block {
            fn $fn_with(&mut self, target: &Block) {
                match (self, target) {
                    (&mut Vec16(ref mut b1), &Vec16(ref b2)) => b1.$fn_with(b2),
                    (this @ &mut Vec16(..), that @ &Vec64(..)) => {
                        this.as_vec64();
                        this.$fn_with(that)
                    }

                    (&mut Vec64(ref mut b1), &Vec16(ref b2)) => b1.$fn_with(b2),
                    (&mut Vec64(ref mut b1), &Vec64(ref b2)) => b1.$fn_with(b2),
                    (&mut Vec64(ref mut b1), &Rle16(ref b2)) => b1.$fn_with(b2),

                    (&mut Rle16(ref mut b1), &Rle16(ref b2)) => b1.$fn_with(b2),

                    (this, that) => {
                        this.as_vec64();
                        this.intersection_with(that);
                    }
                }
            }
        }

    }
}
impl_mutop!(UnionWith, union_with);
impl_mutop!(DifferenceWith, difference_with);
impl_mutop!(SymmetricDifferenceWith, symmetric_difference_with);

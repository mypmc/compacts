use ops::*;

use Vec16;
use self::Vec16::*;

macro_rules! impl_op {
    ( $op:ident, $fn:ident, $fn_with:ident ) => {
        impl<'a, 'b> $op<&'b Vec16> for &'a Vec16 {
            type Output = Vec16;
            fn $fn(self, that: &Vec16) -> Self::Output {
                match (self, that) {
                    (this @ &Seq16(..), that @ &Seq16(..)) => {
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

impl<'a> IntersectionWith<&'a Vec16> for Vec16 {
    fn intersection_with(&mut self, target: &Vec16) {
        match (self, target) {
            (&mut Seq16(ref mut b1), &Seq16(ref b2)) => b1.intersection_with(b2),
            (&mut Seq16(ref mut b1), &Seq64(ref b2)) => b1.intersection_with(b2),

            (&mut Seq64(ref mut b1), &Seq16(ref b2)) => b1.intersection_with(b2),
            (&mut Seq64(ref mut b1), &Seq64(ref b2)) => b1.intersection_with(b2),
            (&mut Seq64(ref mut b1), &Rle16(ref b2)) => b1.intersection_with(b2),

            (&mut Rle16(ref mut b1), &Rle16(ref b2)) => b1.intersection_with(b2),

            (this, that) => {
                this.as_seq64();
                this.intersection_with(that);
            }
        }
    }
}

macro_rules! impl_mutop {
    ( $op:ident, $fn_with:ident ) => {
        impl<'a> $op<&'a Vec16> for Vec16 {
            fn $fn_with(&mut self, target: &Vec16) {
                match (self, target) {
                    (&mut Seq16(ref mut b1), &Seq16(ref b2)) => b1.$fn_with(b2),
                    (this @ &mut Seq16(..), that @ &Seq64(..)) => {
                        this.as_seq64();
                        this.$fn_with(that)
                    }

                    (&mut Seq64(ref mut b1), &Seq16(ref b2)) => b1.$fn_with(b2),
                    (&mut Seq64(ref mut b1), &Seq64(ref b2)) => b1.$fn_with(b2),
                    (&mut Seq64(ref mut b1), &Rle16(ref b2)) => b1.$fn_with(b2),

                    (&mut Rle16(ref mut b1), &Rle16(ref b2)) => b1.$fn_with(b2),

                    (this, that) => {
                        this.as_seq64();
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

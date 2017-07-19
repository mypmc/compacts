use std::u16;

use pair::*;
use super::*;
use self::range::*;

static LHS: &Ranges = &[3...5, 10...13, 18...19, 100...120];
static RHS: &Ranges = &[2...3, 6...9, 12...14, 17...21, 200...1000];

static NULL: &Ranges = &[];
static FULL: &Ranges = &[0...u16::MAX];
static ONE1: &Ranges = &[1...(u16::MAX / 2)];
static ONE2: &Ranges = &[(u16::MAX / 2)...(u16::MAX - 1)];

#[test]
fn two_fold() {

    assert_eq!(
        TwoFold::new(LHS, RHS).collect::<Vec<BelongTo<u32>>>(),
        vec![
            BelongTo::Rhs(2..3),
            BelongTo::Both(3..4),
            BelongTo::Lhs(4..6),
            BelongTo::Rhs(6..10),
            BelongTo::Lhs(10..12),
            BelongTo::Both(12..14),
            BelongTo::Rhs(14..15),
            BelongTo::None(15..17),
            BelongTo::Rhs(17..18),
            BelongTo::Both(18..20),
            BelongTo::Rhs(20..22),
            BelongTo::None(22..100),
            BelongTo::Lhs(100..121),
            BelongTo::None(121..200),
            BelongTo::Rhs(200..1001),
        ]
    );

    assert_eq!(
        TwoFold::new(NULL, RHS).collect::<Vec<BelongTo<u32>>>(),
        vec![
            BelongTo::Rhs(2..4),
            BelongTo::None(4..6),
            BelongTo::Rhs(6..10),
            BelongTo::None(10..12),
            BelongTo::Rhs(12..15),
            BelongTo::None(15..17),
            BelongTo::Rhs(17..22),
            BelongTo::None(22..200),
            BelongTo::Rhs(200..1001),
        ]
    );

    assert_eq!(
        TwoFold::new(LHS, NULL).collect::<Vec<BelongTo<u32>>>(),
        vec![
            BelongTo::Lhs(3..6),
            BelongTo::None(6..10),
            BelongTo::Lhs(10..14),
            BelongTo::None(14..18),
            BelongTo::Lhs(18..20),
            BelongTo::None(20..100),
            BelongTo::Lhs(100..121),
        ]
    );

    assert_eq!(
        TwoFold::new(FULL, RHS).collect::<Vec<BelongTo<u32>>>(),
        vec![
            BelongTo::Lhs(0..2),
            BelongTo::Both(2..4),
            BelongTo::Lhs(4..6),
            BelongTo::Both(6..10),
            BelongTo::Lhs(10..12),
            BelongTo::Both(12..15),
            BelongTo::Lhs(15..17),
            BelongTo::Both(17..22),
            BelongTo::Lhs(22..200),
            BelongTo::Both(200..1001),
            BelongTo::Lhs(1001..65536),
        ]
    );

    assert_eq!(
        TwoFold::new(LHS, FULL).collect::<Vec<BelongTo<u32>>>(),
        vec![
            BelongTo::Rhs(0..3),
            BelongTo::Both(3..6),
            BelongTo::Rhs(6..10),
            BelongTo::Both(10..14),
            BelongTo::Rhs(14..18),
            BelongTo::Both(18..20),
            BelongTo::Rhs(20..100),
            BelongTo::Both(100..121),
            BelongTo::Rhs(121..65536),
        ]
    );

    let a1 = &[0...1, 3...5, 12...16, 18...19];
    let a2 = &[0...0, 3...8, 10...13, 15...15, 19...19];

    assert_eq!(
        TwoFold::new(a1, a2).collect::<Vec<BelongTo<u32>>>(),
        vec![
            BelongTo::Both(0..1),
            BelongTo::Lhs(1..2),
            BelongTo::None(2..3),
            BelongTo::Both(3..6),
            BelongTo::Rhs(6..9),
            BelongTo::None(9..10),
            BelongTo::Rhs(10..12),
            BelongTo::Both(12..14),
            BelongTo::Lhs(14..15),
            BelongTo::Both(15..16),
            BelongTo::Lhs(16..17),
            BelongTo::None(17..18),
            BelongTo::Lhs(18..19),
            BelongTo::Both(19..20),
        ]
    );

    assert_eq!(
        TwoFold::new(a2, a1).collect::<Vec<BelongTo<u32>>>(),
        vec![
            BelongTo::Both(0..1),
            BelongTo::Rhs(1..2),
            BelongTo::None(2..3),
            BelongTo::Both(3..6),
            BelongTo::Lhs(6..9),
            BelongTo::None(9..10),
            BelongTo::Lhs(10..12),
            BelongTo::Both(12..14),
            BelongTo::Rhs(14..15),
            BelongTo::Both(15..16),
            BelongTo::Rhs(16..17),
            BelongTo::None(17..18),
            BelongTo::Rhs(18..19),
            BelongTo::Both(19..20),
        ]
    );
}

macro_rules! test_associativity {
    ( $x:expr, $y:expr, $z:expr, $fn:ident ) => {
        let x = &Rle16::from($x);
        let y = &Rle16::from($y);
        let z = &Rle16::from($z);
        let r1 = x.intersection(y).intersection(z);
        let r2 = x.intersection(&y.intersection(z));
        assert_eq!(r1.ranges, r2.ranges);
        assert_eq!(r1.weight, r2.weight);
    }
}

macro_rules! test_commutativity {
    ( $x:expr, $y:expr, $fn:ident ) => {
        let (w1, v1) = repair(TwoFold::new($x, $y).$fn());
        let (w2, v2) = repair(TwoFold::new($y, $x).$fn());
        assert_eq!(w1, w2);
        assert_eq!(v1, v2);
    }
}

#[test]
fn rle16_intersection() {
    test_associativity!(LHS, RHS, FULL, intersection);
    test_associativity!(LHS, RHS, FULL, intersection);
    test_associativity!(LHS, RHS, ONE1, intersection);
    test_associativity!(RHS, ONE1, ONE2, intersection);
    test_associativity!(RHS, NULL, ONE2, intersection);
    test_associativity!(FULL, NULL, ONE2, intersection);

    test_commutativity!(LHS, RHS, intersection);
    test_commutativity!(NULL, FULL, intersection);
    test_commutativity!(LHS, NULL, intersection);
    test_commutativity!(LHS, FULL, intersection);
    test_commutativity!(ONE1, ONE2, intersection);

    let a1 = &[0...1, 3...5, 12...16, 18...19];
    let a2 = &[0...0, 3...8, 10...13, 15...15, 19...19];
    test_commutativity!(a1, a2, intersection);
}

#[test]
fn rle16_union() {
    test_associativity!(LHS, RHS, FULL, union);
    test_associativity!(LHS, RHS, FULL, union);
    test_associativity!(LHS, RHS, ONE1, union);
    test_associativity!(RHS, ONE1, ONE2, union);
    test_associativity!(RHS, NULL, ONE2, union);
    test_associativity!(FULL, NULL, ONE2, union);

    test_commutativity!(LHS, RHS, union);
    test_commutativity!(NULL, FULL, union);
    test_commutativity!(LHS, NULL, union);
    test_commutativity!(LHS, FULL, union);
    test_commutativity!(ONE1, ONE2, union);
}

#[test]
fn rle16_symmetric_difference() {
    test_associativity!(LHS, RHS, FULL, symmetric_difference);
    test_associativity!(LHS, RHS, FULL, symmetric_difference);
    test_associativity!(LHS, RHS, ONE1, symmetric_difference);
    test_associativity!(RHS, ONE1, ONE2, symmetric_difference);
    test_associativity!(RHS, NULL, ONE2, symmetric_difference);
    test_associativity!(FULL, NULL, ONE2, symmetric_difference);

    test_commutativity!(LHS, RHS, symmetric_difference);
    test_commutativity!(NULL, FULL, symmetric_difference);
    test_commutativity!(LHS, NULL, symmetric_difference);
    test_commutativity!(LHS, FULL, symmetric_difference);
    test_commutativity!(ONE1, ONE2, symmetric_difference);
}

#[test]
fn rle16_test_ops() {
    use std::iter::FromIterator;
    let rle1 = &Rle16::from_iter(vec![0u16, 1, 3, 4, 5, 12, 13, 14, 15, 16, 18, 19]);
    let rle2 = &Rle16::from_iter(vec![0u16, 3, 4, 5, 6, 7, 8, 10, 11, 12, 13, 15, 19]);
    println!("{:?}", rle1);
    println!("{:?}", rle2);

    let i1 = rle1.intersection(rle2);
    let i2 = rle2.intersection(rle1);

    assert_eq!(i1, i2);
}

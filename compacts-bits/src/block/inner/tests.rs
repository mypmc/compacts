use std::mem;
use std::u16;
use super::*;

#[test]
fn inner_size_of_repr() {
    let size = mem::size_of::<Seq16>();
    println!("Vec16 {:?}", size);
    let size = mem::size_of::<Seq64>();
    println!("Vec64 {:?}", size);
    let size = mem::size_of::<Rle16>();
    println!("Rle16 {:?}", size);
}

#[test]
fn inner_rle16() {
    use std::u16;
    let mut vec16 = Seq16::new();
    let mut vec64 = Seq64::new();

    let range1 = 0...6666;
    let range2 = 10000...12345;
    let range3 = (u16::MAX - 3734)...u16::MAX;
    let result = vec![
        range1.clone(),
        range2.clone(),
        23456...23456,
        range3.clone(),
    ];

    {
        for i in range1 {
            vec16.insert(i);
            vec64.insert(i);
        }

        let runlen1 = vec16.count_rle();
        let runlen2 = vec64.count_rle();
        assert_eq!(runlen1, runlen2);
        assert_eq!(1, runlen1);
    }

    {
        for i in range2 {
            vec16.insert(i);
            vec64.insert(i);
        }

        let runlen1 = vec16.count_rle();
        let runlen2 = vec64.count_rle();
        assert_eq!(runlen1, runlen2);
        assert_eq!(2, runlen1);
    }

    {
        vec16.insert(23456);
        vec64.insert(23456);

        let runlen1 = vec16.count_rle();
        let runlen2 = vec64.count_rle();
        assert_eq!(runlen1, runlen2);
        assert_eq!(3, runlen2);
    }

    {
        for i in range3 {
            vec16.insert(i);
            vec64.insert(i);
        }

        let runlen1 = vec16.count_rle();
        let runlen2 = vec64.count_rle();
        assert_eq!(runlen1, runlen2);
        assert_eq!(4, runlen1);
    }

    let rle16_1 = Rle16::from(&vec16);
    let rle16_2 = Rle16::from(&vec64);

    assert!(rle16_1.contains(0));
    assert!(rle16_1.contains(12345));
    assert!(rle16_1.contains(23456));
    assert!(rle16_1.contains(u16::MAX));

    assert!(!rle16_1.contains(7000));
    assert!(!rle16_1.contains(13000));
    assert!(!rle16_1.contains(23455));
    assert!(!rle16_1.contains(23457));

    assert_eq!(rle16_1.weight, rle16_2.weight);
    assert_eq!(rle16_1.weight, vec16.weight);
    assert_eq!(rle16_1.weight, vec64.weight);
    assert_eq!(rle16_1.ranges, result);
    assert_eq!(rle16_2.ranges, result);

    assert_eq!(Seq16::from(&rle16_1), vec16);
    assert_eq!(Seq64::from(&rle16_2), vec64);

    println!("Rle16 {:?}", rle16_1);
    println!("Rle16 {:?}", rle16_2);
}

/*
_,_,_,3,4,5,_,_,_,_,10,11,12,13,__,__,__,__,18,19,__,__,...
_,_,2,3,_,_,6,7,8,9,__,__,12,13,14,__,__,17,18,19,20,21,...
*/
static LHS: &[::std::ops::RangeInclusive<u16>] = &[3...5, 10...13, 18...19, 100...120];
static RHS: &[::std::ops::RangeInclusive<u16>] = &[2...3, 6...9, 12...14, 17...21, 200...1000];

static NULL: &[::std::ops::RangeInclusive<u16>] = &[];
static ONE1: &[::std::ops::RangeInclusive<u16>] = &[1...(u16::MAX / 2)]; // 1...32767
// 32767...65534
static ONE2: &[::std::ops::RangeInclusive<u16>] = &[(u16::MAX / 2)...(u16::MAX - 1)];

static FULL: &[::std::ops::RangeInclusive<u16>] = &[0...u16::MAX]; // 0...65536

#[test]
fn inner_range_folding() {
    // note: result are exclusive
    let want = vec![
        range::BelongTo::Rhs(2..3),
        range::BelongTo::Both(3..4),
        range::BelongTo::Lhs(4..6),
        range::BelongTo::Rhs(6..10),
        range::BelongTo::Lhs(10..12),
        range::BelongTo::Both(12..14),
        range::BelongTo::Rhs(14..15),
        range::BelongTo::None(15..17),
        range::BelongTo::Rhs(17..18),
        range::BelongTo::Both(18..20),
        range::BelongTo::Rhs(20..22),
        range::BelongTo::None(22..100),
        range::BelongTo::Lhs(100..121),
        range::BelongTo::None(121..200),
        range::BelongTo::Rhs(200..1001),
    ];

    for (g, w) in range::Folding::new(LHS, RHS).zip(want) {
        assert_eq!(g, w);
        println!("{:?}", g);
    }
}

#[test]
#[allow(unused_variables)]
fn inner_rle16_intersection() {
    use self::range::*;

    let (w, vec) = range::repair(Folding::new(LHS, RHS).intersection());
    for g in vec {
        println!("Intersection1 {:?}", g);
    }

    let (w, vec) = range::repair(Folding::new(NULL, RHS).intersection());
    for g in vec {
        println!("Intersection2 {:?}", g);
    }

    let (w, vec) = range::repair(Folding::new(ONE1, ONE2).intersection());
    for g in vec {
        println!("Intersection3 {:?}", g);
    }

    let (w, vec) = range::repair(Folding::new(NULL, FULL).intersection());
    for g in vec {
        println!("Intersection4 {:?}", g);
    }
}

#[test]
#[allow(unused_variables)]
fn inner_rle16_union() {
    use self::range::*;

    let (w, vec) = range::repair(Folding::new(LHS, RHS).union());
    for g in &vec {
        println!("Union1 {:?}", g);
    }

    let (w, vec) = range::repair(Folding::new(NULL, RHS).union());
    for g in &vec {
        println!("Union2 {:?}", g);
    }

    let (w, vec) = range::repair(Folding::new(ONE1, ONE2).union());
    assert_eq!(w, 65534, "weight {:?}", w);
    for g in &vec {
        println!("Union3 {:?}", g);
    }

    let (w, vec) = range::repair(Folding::new(NULL, FULL).union());
    assert_eq!(w, 65536, "weight {:?}", w);
    for g in &vec {
        println!("Union4 {:?}", g);
    }
}

#[test]
#[allow(unused_variables)]
fn inner_rle16_difference() {
    use self::range::*;

    let (w, vec) = range::repair(Folding::new(LHS, RHS).difference());
    for g in vec {
        println!("Difference1 {:?}", g);
    }

    let (w, vec) = range::repair(Folding::new(NULL, RHS).difference());
    assert_eq!(w, 0, "weight {:?}", w);
    for g in vec {
        println!("Difference2 {:?}", g);
    }

    let (w, vec) = range::repair(Folding::new(ONE1, ONE2).difference());
    for g in vec {
        println!("Difference3 {:?}", g);
    }

    let (w, vec) = range::repair(Folding::new(NULL, FULL).difference());
    assert_eq!(w, 0, "weight {:?}", w);
    for g in vec {
        println!("Difference4 {:?}", g);
    }
}

#[test]
#[allow(unused_variables)]
fn inner_rle16_symmetric_difference() {
    use self::range::*;

    let (w, vec) = range::repair(Folding::new(LHS, RHS).symmetric_difference());
    for g in vec {
        println!("SymmetricDifference1 {:?}", g);
    }

    let (w, vec) = range::repair(Folding::new(NULL, RHS).symmetric_difference());
    for g in vec {
        println!("SymmetricDifference2 {:?}", g);
    }

    let (w, vec) = range::repair(Folding::new(ONE1, ONE2).symmetric_difference());
    for g in vec {
        println!("SymmetricDifference3 {:?}", g);
    }

    let (w, vec) = range::repair(Folding::new(NULL, FULL).symmetric_difference());
    for g in vec {
        println!("SymmetricDifference4 {:?}", g);
    }
}

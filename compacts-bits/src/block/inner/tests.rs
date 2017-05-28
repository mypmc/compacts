use std::mem;
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

    let range1 = 0...4000;
    let range2 = 10000...10002;
    let range3 = (u16::MAX - 3734)...u16::MAX;
    let result = vec![range1.clone(),
                      range2.clone(),
                      23456...23456,
                      range3.clone()];

    for i in range1 {
        vec16.insert(i);
        vec64.insert(i);
    }
    for i in range2 {
        vec16.insert(i);
        vec64.insert(i);
    }
    vec16.insert(23456);
    vec64.insert(23456);
    for i in range3 {
        vec16.insert(i);
        vec64.insert(i);
    }

    let rle16_1 = Rle16::from(&vec16);
    let rle16_2 = Rle16::from(&vec64);

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

fn set_range(idx: u16, len: u16) -> (u64, u64) {
    let end = len - 1 + idx;
    let x = !0 << idx % 64;
    let y = !0 >> ((-(end as i64)) as u64 % 64);
    return (x, y);
}

#[test]
fn test_set_range() {
    let (x, y) = set_range(0, 1);
    println!("{:064b}\n{:064b}\n{:064b}\n", x, y, x & y);
    let (x, y) = set_range(0, 2);
    println!("{:064b}\n{:064b}\n{:064b}\n", x, y, x & y);
    let (x, y) = set_range(2, 10);
    println!("{:064b}\n{:064b}\n{:064b}\n", x, y, x & y);
}

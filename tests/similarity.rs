// #[macro_use(bits)]
// extern crate compacts;

// use compacts::bit::{ops::*, Words};

// struct Jaccard;
// impl Jaccard {
//     fn similarity(x: &Words<u64>, y: &Words<u64>) -> f64 {
//         let p = x.count1();
//         let q = y.count1();
//         let r = x.and(y).collect::<Words<u64>>();
//         r.count1() as f64 / (p + q - r.count1()) as f64
//     }
// }

// struct Cosine;
// impl Cosine {
//     fn similarity(x: &Words<u64>, y: &Words<u64>) -> f64 {
//         let p = x.count1();
//         let q = y.count1();
//         let r = x.and(y).collect::<Words<u64>>();
//         r.count1() as f64 / ((p * q) as f64).sqrt()
//     }
// }

// #[test]
// fn jaccard_similarity() {
//     let b1 = bits!(0, 2, 3, 4);
//     let b2 = bits!(0, 1, 3, 4, 6);
//     eprintln!("jaccard {}", Jaccard::similarity(&b1, &b2));
// }

// #[test]
// fn cosine_similarity() {
//     let b1 = bits!(0, 2, 3, 4);
//     let b2 = bits!(0, 1, 3, 4, 6);
//     eprintln!("cosine  {}", Cosine::similarity(&b1, &b2));
// }

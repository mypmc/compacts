// #![feature(test)]
// extern crate test;

// use compacts::num::*;
// use rand::prelude::*;
// use test::Bencher;

// #[bench]
// fn rrr_u16_encode(bench: &mut Bencher) {
//     let n = thread_rng().gen::<u16>();
//     bench.iter(|| n.rrr_encode());
// }

// #[bench]
// fn rrr_u16_decode(bench: &mut Bencher) {
//     let (o, c) = thread_rng().gen::<u16>().rrr_encode();
//     bench.iter(|| u16::rrr_decode(o, c));
// }

// #[bench]
// fn rrr_u32_encode(bench: &mut Bencher) {
//     let n = thread_rng().gen::<u32>();
//     bench.iter(|| n.rrr_encode());
// }

// #[bench]
// fn rrr_u32_decode(bench: &mut Bencher) {
//     let (o, c) = thread_rng().gen::<u32>().rrr_encode();
//     bench.iter(|| u32::rrr_decode(o, c));
// }

// #[bench]
// fn rrr_u64_encode(bench: &mut Bencher) {
//     let n = thread_rng().gen::<u64>();
//     bench.iter(|| n.rrr_encode());
// }

// #[bench]
// fn rrr_u64_decode(bench: &mut Bencher) {
//     let (o, c) = thread_rng().gen::<u64>().rrr_encode();
//     bench.iter(|| u64::rrr_decode(o, c));
// }

// #[bench]
// fn rrr_u128_encode(bench: &mut Bencher) {
//     let n = thread_rng().gen::<u128>();
//     bench.iter(|| n.rrr_encode());
// }

// #[bench]
// fn rrr_u128_decode(bench: &mut Bencher) {
//     let (o, c) = thread_rng().gen::<u128>().rrr_encode();
//     bench.iter(|| u128::rrr_decode(o, c));
// }

// extern crate compacts;
// extern crate rand;
// extern crate env_logger;
// #[macro_use]
// extern crate log;

// use rand::Rng;
// use compacts::bits::*;

// macro_rules! bit_vec {
//     ( $size:expr, $end:expr, $rng:expr ) => {
//         {
//             bit_vec!($size, 0, $end, $rng)
//         }
//     };

//     ( $size:expr, $start:expr, $end:expr, $rng:expr ) => {
//         {
//             let mut vec = Map::new();
//             for _ in 0..$size {
//                 let gen = $rng.gen_range($start, $end);
//                 vec.insert(gen);
//             }
//             vec.optimize();
//             vec
//         }
//     };
// }

// #[test]
// fn similarity() {
//     let _ = env_logger::init();
//     let mut rng = rand::thread_rng();

//     let size = (1 << 15) * 7;
//     let maxn = (1 << 16) * 2;

//     let p = &(bit_vec!(size, maxn, rng));
//     let q = &(bit_vec!(size, maxn, rng));

//     let jaccard = {
//         let r = p.intersection(q);
//         r.count_ones() as f64 / (p.count_ones() + q.count_ones() - r.count_ones()) as f64
//     };

//     let dice = {
//         let r = p.intersection(q);
//         (2.0 * (r.count_ones() as f64)) / (p.count_ones() + q.count_ones()) as f64
//     };

//     let simpson = {
//         let r = p.intersection(q);
//         (r.count_ones() as f64) / (p.count_ones() as f64).min(q.count_ones() as f64)
//     };

//     info!("Jaccard  = {:.5?}", jaccard);
//     info!("Dice     = {:.5?}", dice);
//     info!("Simpson  = {:.5?}", simpson);
//     info!("Distance = {:.5?}", 1f64 - jaccard);
// }

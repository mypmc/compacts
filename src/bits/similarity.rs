#![allow(dead_code)]

use super::Set;
use super::PopCount;

pub struct Jaccard;
impl Jaccard {
    pub fn similarity(x: &Set, y: &Set) -> f64 {
        let p = x.count1();
        let q = y.count1();
        let both = x.and(y).collect::<Set>().count1();
        both as f64 / (p + q - both) as f64
    }
}

pub struct Cosine;
impl Cosine {
    pub fn similarity(x: &Set, y: &Set) -> f64 {
        let p = x.count1();
        let q = y.count1();
        let both = x.and(y).collect::<Set>().count1();
        both as f64 / ((p * q) as f64).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jaccard_similarity() {
        let b1 = bitset!(0, 2, 3, 4);
        let b2 = bitset!(0, 1, 3, 4, 6);
        eprintln!("jaccard {}", Jaccard::similarity(&b1, &b2));
    }

    #[test]
    fn test_cosine_similarity() {
        let b1 = bitset!(0, 2, 3, 4);
        let b2 = bitset!(0, 1, 3, 4, 6);
        eprintln!("cosine  {}", Cosine::similarity(&b1, &b2));
    }
}

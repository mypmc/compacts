use super::{Bits, Bucket, Select1};

impl Select1<usize> for Bucket {
    fn select1(&self, c: usize) -> Option<u64> {
        if c as u64 >= self.ones() {
            return None;
        }
        match self {
            &Bucket::Vec(_, ref bits) => bits.get(c).map(|&u| u as u64),
            &Bucket::Map(_, ref bits) => {
                let mut r = c as u64;
                for (i, x) in bits.iter().enumerate() {
                    let w = x.ones();
                    if r < w {
                        let j = i as u64;
                        return Some(Self::BITS_SIZE * j + x.select1(r).unwrap_or(0));
                    }
                    r -= w;
                }
                None
            }
        }
    }
}

/*
impl Select0 for Bucket {
    fn select0(&self, c: usize) -> Option<usize> {
        if c >= Self::SIZE - self.ones() {
            return None;
        }
        match self {
            &Bucket::Vec(_, ref bits) => {
                // [0,2,4,5,6]
            }
            &Bucket::Map(_, ref bits) => {
                let mut r = c;
                for (i, x) in bits.iter().enumerate() {
                    let w = x.zeros();
                    if r < w {
                        return Some(Self::BITS_SIZE * i + x.select0(r).unwrap_or(0));
                    }
                    r -= w;
                }
                None
            }
        }
    }
}
*/

use super::{Bits, Repr, Select1};

impl Select1 for Repr {
    fn select1(&self, c: usize) -> Option<usize> {
        if c >= self.ones() {
            return None;
        }
        match self {
            &Repr::Vec(_, ref bits) => bits.get(c).map(|&u| u as usize),
            &Repr::Map(_, ref bits) => {
                let mut r = c;
                for (i, x) in bits.iter().enumerate() {
                    let w = x.ones();
                    if r < w {
                        return Some(Self::BITS_SIZE * i + x.select1(r).unwrap_or(0));
                    }
                    r -= w;
                }
                None
            }
        }
    }
}

/*
impl Select0 for Repr {
    fn select0(&self, c: usize) -> Option<usize> {
        if c >= Self::SIZE - self.ones() {
            return None;
        }
        match self {
            &Repr::Vec(_, ref bits) => {
                // [0,2,4,5,6]
            }
            &Repr::Map(_, ref bits) => {
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

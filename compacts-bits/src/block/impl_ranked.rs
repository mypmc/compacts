use super::U64_WIDTH;
use super::Block;

use dict::prim::Cast;
use dict::{self, Ranked};

impl dict::Select1<u16> for super::Block {
    fn select1(&self, c: u16) -> Option<u16> {
        if c as u32 >= self.count1() {
            return None;
        }
        match *self {
            Block::Sorted(ref b) => b.vector.get(c as usize).cloned(),
            Block::Mapped(ref b) => {
                let mut rem = c as u32;
                for (i, bit) in b.vector.iter().enumerate() {
                    let ones = bit.count1();
                    if rem < ones {
                        let select = bit.select1(rem).unwrap_or(0);
                        return Some((U64_WIDTH * i) as u16 + select as u16);
                    }
                    rem -= ones;
                }
                None
            }
        }
    }
}

impl dict::Select0<u16> for super::Block {
    fn select0(&self, c: u16) -> Option<u16> {
        let c32 = c as u32;
        if c32 >= self.count0() {
            return None;
        }
        match *self {
            Block::Sorted(..) => {
                let pos = dict::search(&(0..Self::CAPACITY), |i| {
                    Cast::from::<u32>(i)
                        .and_then(|conv: u16| {
                                      let rank = self.rank0(conv);
                                      Cast::from::<u32>(rank)
                                  })
                        .map_or(false, |rank: u16| rank > c)
                });
                if pos < Self::CAPACITY {
                    Some(Cast::from::<u32>(pos).expect("pos < capacity, cast must not failed"))
                } else {
                    None
                }
            }
            Block::Mapped(ref b) => {
                let mut rem = c32;
                for (i, bit) in b.vector.iter().enumerate() {
                    let zeros = bit.count0();
                    if rem < zeros {
                        let select = bit.select0(rem).unwrap_or(0);
                        return Some((U64_WIDTH * i) as u16 + select as u16);
                    }
                    rem -= zeros;
                }
                None
            }
        }
    }
}

impl dict::Ranked<u16> for super::Block {
    type Weight = u32;

    fn size(&self) -> Self::Weight {
        Self::CAPACITY as u32
    }

    fn count1(&self) -> Self::Weight {
        match *self {
            Block::Sorted(ref b) => b.weight,
            Block::Mapped(ref b) => b.weight,
        }
    }

    fn rank1(&self, i: u16) -> Self::Weight {
        if i as u32 >= Self::CAPACITY {
            return self.count1();
        }
        match *self {
            Block::Sorted(ref b) => {
                let vec = &b.vector;
                let k = dict::search(&(0..vec.len()), |j| vec.get(j).map_or(false, |&v| v >= i));
                (if k < vec.len() && vec[k] == i {
                     k + 1
                 } else {
                     k
                 }) as Self::Weight
            }

            Block::Mapped(ref b) => {
                let q = i as usize / U64_WIDTH;
                let r = i as usize % U64_WIDTH;
                let r = r as u32;
                let vec = &b.vector;
                vec.iter().take(q).fold(0, |acc, w| acc + w.count1()) +
                vec.get(q).map_or(0, |w| w.rank1(r))
            }
        }
    }
}

use dict::Ranked;
use pairwise;
use self::pairwise::{Pairwise, PairwiseWith};

use super::{Block, Bucket};

impl<'a> Pairwise<&'a Block> for Block {
    type Output = Block;

    fn intersection(&self, that: &Block) -> Self::Output {
        match (self, that) {
            (this @ &Block::Sorted(..), that @ &Block::Sorted(..)) => {
                let pair = pairwise::intersection(this.iter(), that.iter());
                pair.collect()
            }
            (this, that) => {
                let mut cloned = this.clone();
                cloned.intersection_with(that);
                cloned
            }
        }
    }

    fn union(&self, that: &Block) -> Self::Output {
        match (self, that) {
            (this @ &Block::Sorted(..), that @ &Block::Sorted(..)) => {
                let pair = pairwise::union(this.iter(), that.iter());
                pair.collect()
            }
            (this, that) => {
                let mut cloned = this.clone();
                cloned.union_with(that);
                cloned
            }
        }
    }

    fn difference(&self, that: &Block) -> Self::Output {
        match (self, that) {
            (this @ &Block::Sorted(..), that @ &Block::Sorted(..)) => {
                let pair = pairwise::difference(this.iter(), that.iter());
                pair.collect()
            }

            (this, that) => {
                let mut cloned = this.clone();
                cloned.difference_with(that);
                cloned
            }
        }
    }

    fn symmetric_difference(&self, that: &Block) -> Self::Output {
        match (self, that) {
            (this @ &Block::Sorted(..), that @ &Block::Sorted(..)) => {
                let pair = pairwise::symmetric_difference(this.iter(), that.iter());
                pair.collect()
            }
            (this, that) => {
                let mut cloned = this.clone();
                cloned.symmetric_difference_with(that);
                cloned
            }
        }
    }
}

impl<'a> PairwiseWith<&'a Block> for Block {
    fn intersection_with(&mut self, target: &Block) {
        match (self, target) {
            (&mut Block::Mapped(ref mut b1), &Block::Mapped(ref b2)) => {
                bucket_foreach!(b1 & b2);
            }

            (&mut Block::Mapped(ref mut b1), &Block::Sorted(ref b2)) => {
                let b3 = Bucket::from(b2);
                bucket_foreach!(b1 & b3);
            }

            (&mut Block::Sorted(ref mut b), that @ &Block::Mapped(..)) => {
                let weight = {
                    let mut weight = 0;
                    for i in 0..b.vector.len() {
                        if that.contains(b.vector[i]) {
                            b.vector[weight] = b.vector[i];
                            weight += 1;
                        }
                    }
                    weight
                };
                b.vector.truncate(weight);
                b.weight = weight as u32;
            }

            (this, that) => {
                *this = {
                    let pair = pairwise::intersection(this.iter(), that.iter());
                    pair.collect()
                };
            }
        }
    }

    fn union_with(&mut self, target: &Block) {
        if target.count1() == 0 {
            return;
        }
        match (self, target) {
            (&mut Block::Mapped(ref mut b1), &Block::Mapped(ref b2)) => {
                bucket_foreach!(b1 | b2);
            }

            (&mut Block::Mapped(ref mut b1), &Block::Sorted(ref b2)) => {
                for &bit in &b2.vector[..] {
                    b1.insert(bit);
                }
            }

            (this @ &mut Block::Sorted(..), that @ &Block::Mapped(..)) => {
                this.as_mapped();
                this.union_with(that)
            }

            (this, that) => {
                *this = {
                    let pair = pairwise::union(this.iter(), that.iter());
                    pair.collect()
                };
            }
        }
    }

    fn difference_with(&mut self, target: &Block) {
        match (self, target) {
            (&mut Block::Mapped(ref mut b1), &Block::Mapped(ref b2)) => {
                bucket_foreach!(b1 - b2);
            }
            (&mut Block::Mapped(ref mut b1), &Block::Sorted(ref b2)) => {
                for &item in &b2.vector[..] {
                    b1.remove(item);
                }
            }

            (this @ &mut Block::Sorted(..), that @ &Block::Mapped(..)) => {
                this.as_mapped();
                this.difference_with(that);
            }

            (this, that) => {
                *this = {
                    let pair = pairwise::difference(this.iter(), that.iter());
                    pair.collect()
                };
            }
        }
    }

    fn symmetric_difference_with(&mut self, target: &Block) {
        match (self, target) {
            (&mut Block::Mapped(ref mut b1), &Block::Mapped(ref b2)) => {
                bucket_foreach!(b1 ^ b2);
            }

            (ref mut this @ &mut Block::Mapped(..), &Block::Sorted(ref b)) => {
                for &bit in &b.vector {
                    if this.contains(bit) {
                        this.remove(bit);
                    } else {
                        this.insert(bit);
                    }
                }
            }

            (this @ &mut Block::Sorted(..), that @ &Block::Mapped(..)) => {
                this.as_mapped();
                this.symmetric_difference_with(that)
            }

            (this, that) => {
                *this = {
                    let pair = pairwise::symmetric_difference(this.iter(), that.iter());
                    pair.collect()
                };
            }
        }
    }
}

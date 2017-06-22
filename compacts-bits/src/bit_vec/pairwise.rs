use ops::*;

macro_rules! impl_op {
    ( $op:ident, $fn_name:ident, $fn:ident ) => {
        impl $op<super::BitVec> for super::BitVec {
            type Output = super::BitVec;
            fn $fn_name(self, that: super::BitVec) -> Self::Output {
                let mut this = self;
                this.$fn(&that);
                this
            }
        }
        impl<'r> $op<&'r super::BitVec> for super::BitVec {
            type Output = super::BitVec;
            fn $fn_name(self, that: &super::BitVec) -> Self::Output {
                let mut this = self;
                this.$fn(that);
                this
            }
        }
        impl<'r1, 'r2> $op<&'r2 super::BitVec> for &'r1 super::BitVec {
            type Output = super::BitVec;
            fn $fn_name(self, that: &super::BitVec) -> Self::Output {
                let mut this = self.clone();
                this.$fn(that);
                this
            }
        }

    }
}

impl_op!(Intersection, intersection, intersection_with);
impl_op!(Union, union, union_with);
impl_op!(Difference, difference, difference_with);
impl_op!(
    SymmetricDifference,
    symmetric_difference,
    symmetric_difference_with
);

fn block_or(mut lhs: super::Block, rhs: &super::Block) -> super::Block {
    lhs.union_with(rhs);
    lhs
}

fn block_andnot(mut lhs: super::Block, rhs: &super::Block) -> super::Block {
    lhs.difference_with(rhs);
    lhs
}

fn block_xor(mut lhs: super::Block, rhs: &super::Block) -> super::Block {
    lhs.symmetric_difference_with(rhs);
    lhs
}

impl<'r> ::ops::IntersectionWith<&'r super::BitVec> for super::BitVec {
    fn intersection_with(&mut self, that: &'r super::BitVec) {
        let rms = {
            let mut rms = Vec::with_capacity(self.blocks.len());
            for (key, b) in &mut self.blocks {
                if that.blocks.contains_key(key) {
                    b.intersection_with(&that.blocks[key]);
                    if b.count_ones() != 0 {
                        b.optimize();
                    } else {
                        rms.push(*key);
                    }
                } else {
                    rms.push(*key);
                }
            }
            rms
        };
        for rm in &rms {
            let removed = self.blocks.remove(rm);
            debug_assert!(removed.is_some());
        }
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(map_entry))]
impl<'r> ::ops::UnionWith<&'r super::BitVec> for super::BitVec {
    fn union_with(&mut self, that: &'r super::BitVec) {
        for (&key, b) in &that.blocks {
            let rb = (**b).clone();
            if !self.blocks.contains_key(&key) {
                self.blocks.insert(key, eval!(rb));
                continue;
            }
            let lb = (*self.blocks[&key]).clone();
            self.blocks.insert(key, lazy!(block_or(lb, &rb)));
        }
    }
}

impl<'r> ::ops::DifferenceWith<&'r super::BitVec> for super::BitVec {
    fn difference_with(&mut self, that: &'r super::BitVec) {
        for (&key, b) in &mut self.blocks {
            if !that.blocks.contains_key(&key) {
                continue;
            }
            let lb = (**b).clone();
            let rb = (*that.blocks[&key]).clone();
            *b = lazy!(block_andnot(lb, &rb));
        }
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(map_entry))]
impl<'r> ::ops::SymmetricDifferenceWith<&'r super::BitVec> for super::BitVec {
    fn symmetric_difference_with(&mut self, that: &'r super::BitVec) {
        for (&key, b) in &that.blocks {
            let rb = (**b).clone();
            if !self.blocks.contains_key(&key) {
                self.blocks.insert(key, eval!(rb));
                continue;
            }
            let lb = (*self.blocks[&key]).clone();
            self.blocks.insert(key, lazy!(block_xor(lb, &rb)));
        }
    }
}

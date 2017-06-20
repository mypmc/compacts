use ops::*;
use super::Block;

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

fn union_with(mut lhs: Block, rhs: &Block) -> Block {
    lhs.union_with(rhs);
    lhs
}

fn difference_with(mut lhs: Block, rhs: &Block) -> Block {
    lhs.difference_with(rhs);
    lhs
}

fn symmetric_difference_with(mut lhs: Block, rhs: &Block) -> Block {
    lhs.symmetric_difference_with(rhs);
    lhs
}

impl<'r> ::ops::IntersectionWith<&'r super::BitVec> for super::BitVec {
    fn intersection_with(&mut self, that: &'r super::BitVec) {
        let keys = {
            let mut remove = Vec::with_capacity(self.blocks.len());
            for (key, b) in &mut self.blocks {
                if that.blocks.contains_key(key) {
                    b.intersection_with(&that.blocks[key]);
                    let ones = b.count_ones();
                    if ones == 0 {
                        remove.push(*key);
                        continue;
                    }
                    b.optimize();
                } else {
                    remove.push(*key);
                }
            }
            remove
        };
        for key in keys {
            let removed = self.blocks.remove(&key);
            assert!(removed.is_some());
        }
    }
}

impl<'r> ::ops::UnionWith<&'r super::BitVec> for super::BitVec {
    fn union_with(&mut self, that: &'r super::BitVec) {
        for (&key, thunk) in &that.blocks {
            let rb = (**thunk).clone();
            if !self.blocks.contains_key(&key) {
                self.blocks.insert(key, eval!(rb));
                continue;
            }
            let lb = (*self.blocks[&key]).clone();
            let deferred = lazy!(union_with(lb, &rb));
            self.blocks.insert(key, deferred);
        }
    }
}

impl<'r> ::ops::DifferenceWith<&'r super::BitVec> for super::BitVec {
    fn difference_with(&mut self, that: &'r super::BitVec) {
        let diff = {
            let mut thunks = Vec::with_capacity(64);
            for (&key, thunk) in &self.blocks {
                if !that.blocks.contains_key(&key) {
                    continue;
                }
                let lb = (**thunk).clone();
                let rb = (*that.blocks[&key]).clone();
                let deferred = lazy!(difference_with(lb, &rb));
                thunks.push((key, deferred));
            }
            thunks
        };
        for (k, t) in diff {
            self.blocks.insert(k, t);
        }
    }
}

impl<'r> ::ops::SymmetricDifferenceWith<&'r super::BitVec> for super::BitVec {
    fn symmetric_difference_with(&mut self, that: &'r super::BitVec) {
        for (&key, thunk) in &that.blocks {
            let rb = (**thunk).clone();
            if !self.blocks.contains_key(&key) {
                self.blocks.insert(key, eval!(rb));
                continue;
            }
            let lb = (*self.blocks[&key]).clone();
            let deferred = lazy!(symmetric_difference_with(lb, &rb));
            self.blocks.insert(key, deferred);
        }
    }
}

use ops::*;

macro_rules! impl_op {
    ( $op:ident, $fn_name:ident, $fn:ident ) => {
        impl<'a, 'b> $op<super::BitVec<'b>> for super::BitVec<'a> {
            type Output = super::BitVec<'a>;
            fn $fn_name(self, that: super::BitVec<'b>) -> Self::Output {
                let mut this = self;
                this.$fn(&that);
                this
            }
        }
        impl<'r, 'a, 'b> $op<&'r super::BitVec<'b>> for super::BitVec<'a>
            where 'b: 'r
        {
            type Output = super::BitVec<'a>;
            fn $fn_name(self, that: &super::BitVec<'b>) -> Self::Output {
                let mut this = self;
                this.$fn(that);
                this
            }
        }
        impl<'r1, 'r2, 'a, 'b> $op<&'r2 super::BitVec<'b>> for &'r1 super::BitVec<'a>
            where 'a: 'r1,
        'b: 'r2
        {
            type Output = super::BitVec<'a>;
            fn $fn_name(self, that: &super::BitVec<'b>) -> Self::Output {
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
impl_op!(SymmetricDifference,
         symmetric_difference,
         symmetric_difference_with);

impl<'r, 'a, 'b> ::ops::IntersectionWith<&'r super::BitVec<'b>> for super::BitVec<'a>
    where 'a: 'r,
          'b: 'r
{
    fn intersection_with(&mut self, that: &'r super::BitVec<'b>) {
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

impl<'r, 'a, 'b> ::ops::UnionWith<&'r super::BitVec<'b>> for super::BitVec<'a>
    where 'a: 'r,
          'b: 'r
{
    fn union_with(&mut self, that: &'r super::BitVec<'b>) {
        for (&key, thunk) in &that.blocks {
            let rb = (**thunk).clone();
            if !self.blocks.contains_key(&key) {
                self.blocks.insert(key, eval!(rb));
                continue;
            }
            let mut lb = (*self.blocks[&key]).clone();
            let deferred = lazy!({
                                     lb.union_with(&rb);
                                     lb
                                 });
            self.blocks.insert(key, deferred);
        }
    }
}

impl<'r, 'a, 'b> ::ops::DifferenceWith<&'r super::BitVec<'b>> for super::BitVec<'a>
    where 'a: 'r,
          'b: 'r
{
    fn difference_with(&mut self, that: &'r super::BitVec<'b>) {
        let diff = {
            let mut thunks = Vec::with_capacity(64);
            for (&key, thunk) in &self.blocks {
                if !that.blocks.contains_key(&key) {
                    continue;
                }
                let mut lb = (**thunk).clone();
                let rb = (*that.blocks[&key]).clone();
                let deferred = lazy!({
                                         lb.difference_with(&rb);
                                         lb
                                     });
                thunks.push((key, deferred));
            }
            thunks
        };
        for (k, t) in diff {
            self.blocks.insert(k, t);
        }
    }
}

impl<'r, 'a, 'b> ::ops::SymmetricDifferenceWith<&'r super::BitVec<'b>> for super::BitVec<'a>
    where 'a: 'r,
          'b: 'r
{
    fn symmetric_difference_with(&mut self, that: &'r super::BitVec<'b>) {
        for (&key, thunk) in &that.blocks {
            let rb = (**thunk).clone();
            if !self.blocks.contains_key(&key) {
                self.blocks.insert(key, eval!(rb));
                continue;
            }

            let mut lb = (*self.blocks[&key]).clone();
            let deferred = lazy!({
                                     lb.symmetric_difference_with(&rb);
                                     lb.optimize();
                                     lb
                                 });
            self.blocks.insert(key, deferred);
        }
    }
}

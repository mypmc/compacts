use ops::*;

macro_rules! impl_op {
    ( $op:ident, $fn_name:ident, $fn:ident ) => {
        impl $op<super::Vec32> for super::Vec32 {
            type Output = super::Vec32;
            fn $fn_name(self, that: super::Vec32) -> Self::Output {
                let mut this = self;
                this.$fn(&that);
                this
            }
        }
        impl<'r> $op<&'r super::Vec32> for super::Vec32 {
            type Output = super::Vec32;
            fn $fn_name(self, that: &super::Vec32) -> Self::Output {
                let mut this = self;
                this.$fn(that);
                this
            }
        }
        impl<'r1, 'r2> $op<&'r2 super::Vec32> for &'r1 super::Vec32 {
            type Output = super::Vec32;
            fn $fn_name(self, that: &super::Vec32) -> Self::Output {
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

fn block_or(mut lhs: super::Vec16, rhs: &super::Vec16) -> super::Vec16 {
    lhs.union_with(rhs);
    lhs
}

fn block_andnot(mut lhs: super::Vec16, rhs: &super::Vec16) -> super::Vec16 {
    lhs.difference_with(rhs);
    lhs
}

fn block_xor(mut lhs: super::Vec16, rhs: &super::Vec16) -> super::Vec16 {
    lhs.symmetric_difference_with(rhs);
    lhs
}

impl<'r> ::ops::IntersectionWith<&'r super::Vec32> for super::Vec32 {
    fn intersection_with(&mut self, that: &'r super::Vec32) {
        let rms = {
            let mut rms = Vec::with_capacity(self.vec16s.len());
            for (key, b) in &mut self.vec16s {
                if that.vec16s.contains_key(key) {
                    b.intersection_with(&that.vec16s[key]);
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
            let removed = self.vec16s.remove(rm);
            debug_assert!(removed.is_some());
        }
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(map_entry))]
impl<'r> ::ops::UnionWith<&'r super::Vec32> for super::Vec32 {
    fn union_with(&mut self, that: &'r super::Vec32) {
        for (&key, b) in &that.vec16s {
            let rb = (**b).clone();
            if !self.vec16s.contains_key(&key) {
                self.vec16s.insert(key, eval!(rb));
                continue;
            }
            let lb = (*self.vec16s[&key]).clone();
            self.vec16s.insert(key, lazy!(block_or(lb, &rb)));
        }
    }
}

impl<'r> ::ops::DifferenceWith<&'r super::Vec32> for super::Vec32 {
    fn difference_with(&mut self, that: &'r super::Vec32) {
        for (&key, b) in &mut self.vec16s {
            if !that.vec16s.contains_key(&key) {
                continue;
            }
            let lb = (**b).clone();
            let rb = (*that.vec16s[&key]).clone();
            *b = lazy!(block_andnot(lb, &rb));
        }
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(map_entry))]
impl<'r> ::ops::SymmetricDifferenceWith<&'r super::Vec32> for super::Vec32 {
    fn symmetric_difference_with(&mut self, that: &'r super::Vec32) {
        for (&key, b) in &that.vec16s {
            let rb = (**b).clone();
            if !self.vec16s.contains_key(&key) {
                self.vec16s.insert(key, eval!(rb));
                continue;
            }
            let lb = (*self.vec16s[&key]).clone();
            self.vec16s.insert(key, lazy!(block_xor(lb, &rb)));
        }
    }
}

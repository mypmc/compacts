use ops::*;

macro_rules! impl_op {
    ( $op:ident, $fn_name:ident, $fn:ident ) => {
        impl $op<super::Vec64> for super::Vec64 {
            type Output = super::Vec64;
            fn $fn_name(self, that: super::Vec64) -> Self::Output {
                let mut this = self;
                this.$fn(&that);
                this
            }
        }
        impl<'r> $op<&'r super::Vec64> for super::Vec64 {
            type Output = super::Vec64;
            fn $fn_name(self, that: &super::Vec64) -> Self::Output {
                let mut this = self;
                this.$fn(that);
                this
            }
        }
        impl<'r1, 'r2> $op<&'r2 super::Vec64> for &'r1 super::Vec64 {
            type Output = super::Vec64;
            fn $fn_name(self, that: &super::Vec64) -> Self::Output {
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

impl<'r> IntersectionWith<&'r super::Vec64> for super::Vec64 {
    fn intersection_with(&mut self, that: &'r super::Vec64) {

        let keys_to_remove = {
            let mut keys = Vec::with_capacity(self.vec32s.len());
            for (key, vec) in &mut self.vec32s {
                if that.vec32s.contains_key(key) {
                    vec.intersection_with(&that.vec32s[key]);
                    if vec.count_ones() != 0 {
                        vec.optimize();
                    } else {
                        keys.push(*key);
                    }
                } else {
                    keys.push(*key);
                }
            }
            keys
        };

        for key in keys_to_remove {
            let removed = self.vec32s.remove(&key);
            debug_assert!(removed.is_some());
        }
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(map_entry))]
impl<'r> UnionWith<&'r super::Vec64> for super::Vec64 {
    fn union_with(&mut self, that: &'r super::Vec64) {
        for (&key, vec) in &that.vec32s {
            if !self.vec32s.contains_key(&key) {
                self.vec32s.insert(key, vec.clone());
                continue;
            }
            let mut lb = self.vec32s[&key].clone();
            lb.union_with(vec);
            self.vec32s.insert(key, lb);
        }
    }
}

impl<'r> DifferenceWith<&'r super::Vec64> for super::Vec64 {
    fn difference_with(&mut self, that: &'r super::Vec64) {
        for (&key, vec) in &mut self.vec32s {
            if !that.vec32s.contains_key(&key) {
                continue;
            }
            let lb = vec.clone();
            let rb = &that.vec32s[&key];
            *vec = lb.difference(rb);
        }
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(map_entry))]
impl<'r> SymmetricDifferenceWith<&'r super::Vec64> for super::Vec64 {
    fn symmetric_difference_with(&mut self, that: &'r super::Vec64) {
        for (&key, vec) in &that.vec32s {
            if !self.vec32s.contains_key(&key) {
                self.vec32s.insert(key, vec.clone());
                continue;
            }
            let mut b = self.vec32s[&key].clone();
            b.symmetric_difference_with(vec);
            self.vec32s.insert(key, b);
        }
    }
}

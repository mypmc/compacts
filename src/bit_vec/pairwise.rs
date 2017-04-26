use std::iter::{Fuse, Peekable, ExactSizeIterator};
use std::cmp::{self, Ordering};
use bits::PairwiseWith;
use dict::Ranked;
use super::BitVec;

impl<'r, 'a, 'b> PairwiseWith<&'r BitVec<'b>> for BitVec<'a>
    where 'a: 'r,
          'b: 'r
{
    fn intersection_with(&mut self, that: &'r BitVec<'b>) {
        let keys = {
            let mut remove = Vec::with_capacity(self.blocks.len());
            for (key, b) in self.blocks.iter_mut() {
                if that.blocks.contains_key(key) {
                    b.intersection_with(&that.blocks[key]);
                    let ones = b.count1();
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

    fn union_with(&mut self, that: &'r BitVec<'b>) {
        for (&key, thunk) in that.blocks.iter() {
            let rb = (**thunk).clone();
            if !self.blocks.contains_key(&key) {
                self.blocks.insert(key, eval!(rb));
                continue;
            }
            let mut lb = (**self.blocks.get(&key).unwrap()).clone();
            let deferred = lazy!(move {
                trace!("evaluate UNION of <{:?} {:?}> key={:?}", lb, rb, key);
                lb.union_with(&rb);
                lb.optimize();
                lb
            });
            self.blocks.insert(key, deferred);
        }
    }

    fn difference_with(&mut self, that: &'r BitVec<'b>) {
        let diff = {
            let mut thunks = Vec::with_capacity(64);
            for (&key, thunk) in self.blocks.iter() {
                if !that.blocks.contains_key(&key) {
                    continue;
                }
                let mut lb = (**thunk).clone();
                let rb = (**that.blocks.get(&key).unwrap()).clone();
                let deferred = lazy!(move {
                    trace!("evaluate DIFFERENCE of <{:?} {:?}> key={:?}", lb, rb, key);
                    lb.difference_with(&rb);
                    lb.optimize();
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

    fn symmetric_difference_with(&mut self, that: &'r BitVec<'b>) {
        for (&key, thunk) in that.blocks.iter() {
            let rb = (**thunk).clone();
            if !self.blocks.contains_key(&key) {
                self.blocks.insert(key, eval!(rb));
                continue;
            }

            let mut lb = (**self.blocks.get(&key).unwrap()).clone();
            let deferred = lazy!(move {
                trace!("evaluate SYMMETRIC_DIFFERENCE of <{:?} {:?}> key={:?}", lb, rb, key);
                lb.symmetric_difference_with(&rb);
                lb.optimize();
                lb
            });
            self.blocks.insert(key, deferred);
        }
    }
}

macro_rules! define_pair {
    ( $( ( $fn:ident, $op:ident ) ),* ) => ($(
        /// Struct for a slow but generic pairwise operations.
        pub struct $op<I1, I2, T>
            where I1: Iterator<Item = T>,
                  I2: Iterator<Item = T>
        {
            lhs: Peekable<Fuse<I1>>,
            rhs: Peekable<Fuse<I2>>,
        }

        /// Assume that I1 and I2 are sorted.
        pub fn $fn<I1, I2, T>(x: I1, y: I2) -> $op<I1, I2, T>
            where I1: Iterator<Item = T> + ExactSizeIterator,
                  I2: Iterator<Item = T> + ExactSizeIterator,
                  T: Ord
        {
            $op {lhs: x.fuse().peekable(), rhs: y.fuse().peekable()}
        }
    )*);
}

define_pair!((intersection, Intersection),
             (union, Union),
             (difference, Difference),
             (symmetric_difference, SymmetricDifference));

/// Compare `a` and `b`, but return `s` if a is None and `l` if b is None
fn comparing<T: Ord>(a: Option<T>,
                     b: Option<T>,
                     x: cmp::Ordering,
                     y: cmp::Ordering)
                     -> cmp::Ordering {
    match (a, b) {
        (None, _) => x,
        (_, None) => y,
        (Some(ref a1), Some(ref b1)) => a1.cmp(b1),
    }
}

impl<I1, I2, T> Iterator for Intersection<I1, I2, T>
    where I1: Iterator<Item = T> + ExactSizeIterator,
          I2: Iterator<Item = T> + ExactSizeIterator,
          T: Ord
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        use self::Ordering::{Less, Equal, Greater};
        loop {
            let compared = {
                let x = self.lhs.peek();
                let y = self.rhs.peek();
                x.and_then(|x1| y.map(|y1| x1.cmp(&y1)))
            };
            match compared {
                None => return None,
                Some(Less) => {
                    self.lhs.next();
                }
                Some(Equal) => {
                    self.rhs.next();
                    return self.lhs.next();
                }
                Some(Greater) => {
                    self.rhs.next();
                }
            }
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(cmp::min(self.lhs.len(), self.rhs.len())))
    }
}

impl<I1, I2, T> Iterator for Union<I1, I2, T>
    where I1: Iterator<Item = T> + ExactSizeIterator,
          I2: Iterator<Item = T> + ExactSizeIterator,
          T: Ord
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        use self::Ordering::{Less, Equal, Greater};
        loop {
            match comparing(self.lhs.peek(), self.rhs.peek(), Greater, Less) {
                Less => return self.lhs.next(),
                Equal => {
                    self.rhs.next();
                    return self.lhs.next();
                }
                Greater => return self.rhs.next(),
            }
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let x_len = self.lhs.len();
        let y_len = self.rhs.len();
        (cmp::max(x_len, y_len), Some(x_len + y_len))
    }
}

impl<I1, I2, T> Iterator for Difference<I1, I2, T>
    where I1: Iterator<Item = T> + ExactSizeIterator,
          I2: Iterator<Item = T> + ExactSizeIterator,
          T: Ord
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        use self::Ordering::{Less, Equal, Greater};
        loop {
            let compaed = comparing(self.lhs.peek(), self.rhs.peek(), Less, Less);
            match compaed {
                Less => return self.lhs.next(),
                Equal => {
                    self.lhs.next();
                    self.rhs.next();
                }
                Greater => {
                    self.rhs.next();
                }
            }
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let x_len = self.lhs.len();
        let y_len = self.rhs.len();
        (x_len.saturating_sub(y_len), Some(x_len))
    }
}

impl<I1, I2, T> Iterator for SymmetricDifference<I1, I2, T>
    where I1: Iterator<Item = T> + ExactSizeIterator,
          I2: Iterator<Item = T> + ExactSizeIterator,
          T: Ord
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        use self::Ordering::{Less, Equal, Greater};
        loop {
            match comparing(self.lhs.peek(), self.rhs.peek(), Greater, Less) {
                Less => return self.lhs.next(),
                Equal => {
                    self.lhs.next();
                    self.rhs.next();
                }
                Greater => return self.rhs.next(),
            }
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.lhs.len() + self.rhs.len()))
    }
}

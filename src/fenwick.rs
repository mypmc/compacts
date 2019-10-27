#![allow(dead_code, unused_imports)]

use std::{
    cmp,
    convert::identity,
    iter::{from_fn, Sum},
    ops::{AddAssign, SubAssign},
};

use crate::num::{Int, Word};

/// 1-based FenwickTree (or BinaryIndexedTree)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FenwickTree<T> {
    pub(crate) tree: Vec<T>,
}

#[inline]
pub(crate) fn next_fwd(pos: usize) -> usize {
    pos + (pos & (-(pos as isize) as usize))
}

#[inline]
pub(crate) fn next_bwd(pos: usize) -> usize {
    pos - (pos & (-(pos as isize) as usize))
}

pub(crate) fn fwd_links(pos: usize, max: usize) -> impl Iterator<Item = usize> {
    let mut next = pos + 1;
    from_fn(move || {
        if next < max {
            let curr = next;
            next = next_fwd(next);
            Some(curr)
        } else {
            None
        }
    })
}

pub(crate) fn bwd_links(pos: usize) -> impl Iterator<Item = usize> {
    let mut next = pos;
    from_fn(move || {
        if next > 0 {
            let curr = next;
            next = next_bwd(next);
            Some(curr)
        } else {
            None
        }
    })
}

impl<T: Copy> FenwickTree<T> {
    // Assume that `ident <> ident == ident`.
    pub fn init(size: usize, ident: T) -> Self {
        FenwickTree {
            tree: vec![ident; size + 1],
        }
    }

    // Assume that `T::default()` is the identity.
    pub fn with_default(size: usize) -> Self
    where
        T: Default,
    {
        FenwickTree::init(size, T::default())
    }

    pub fn len(&self) -> usize {
        self.tree.len() - 1 // 0 is sentinel
    }

    /// Adds delta to value at `i`.
    pub fn add<U: Copy>(&mut self, pos: usize, delta: U)
    where
        T: AddAssign<U>,
    {
        for i in fwd_links(pos, self.tree.len()) {
            self.tree[i] += delta;
        }
    }

    /// Subtracts delta from value at `i`.
    pub fn sub<U: Copy>(&mut self, pos: usize, delta: U)
    where
        T: SubAssign<U>,
    {
        for i in fwd_links(pos, self.tree.len()) {
            self.tree[i] -= delta;
        }
    }

    // /// Returns values corresponding to the given index `[0, pos)`.
    // #[inline]
    // pub fn values<'a>(&'a self, pos: usize) -> impl Iterator<Item = T> + 'a {
    //     bwd_links(pos).map(move |i| self.tree[i])
    // }

    /// Returns the sum within `[0, pos)`.
    #[inline]
    pub fn sum<A: Sum<T>>(&self, pos: usize) -> A {
        bwd_links(pos).map(|i| self.tree[i]).sum()
    }

    #[inline]
    pub fn sum_by<A, B, F>(&self, pos: usize, mut f: F) -> A
    where
        A: Sum<B>,
        F: FnMut(T) -> B,
    {
        bwd_links(pos).map(|i| f(self.tree[i])).sum()
    }
}

impl<T: Copy> FenwickTree<T> {
    /// Finds the lowest index that satisfies `self.sum(i) >= sum` and returns the leftover.
    #[inline]
    pub fn search(&self, sum: T) -> Result<usize, usize>
    where
        T: PartialOrd + SubAssign,
    {
        self.search_by(sum, identity)
    }

    pub(crate) fn search_by<U, F>(&self, mut sum: U, mut f: F) -> Result<usize, usize>
    where
        U: PartialOrd + SubAssign,
        F: FnMut(T) -> U,
    {
        let mut pos = 0;
        let mut k = prev_power_of_two(self.tree.len());
        debug_assert!(k >= 1); // because of sentinel, tree's min length is 1, not 0

        while k > 0 {
            if pos + k < self.tree.len() {
                let tip = f(self.tree[pos + k]);
                if sum > tip {
                    sum -= tip;
                    pos += k;
                }
            }
            k /= 2;
        }

        if pos < self.len() {
            Ok(pos)
        } else {
            Err(pos)
        }
    }
}

impl<T: Int> FenwickTree<T> {
    /// Gets an actual value at `i`.
    #[inline]
    pub fn get(&self, i: usize) -> T {
        self.sum::<T>(i + 1) - self.sum::<T>(i)
    }

    /// Sets a new value at `i`.
    pub fn set(&mut self, i: usize, val: T) {
        let cur = self.get(i);
        if cur <= val {
            self.add(i, val - cur);
        } else {
            self.sub(i, cur - val);
        }
    }
}

impl<T, U> Extend<U> for FenwickTree<T>
where
    T: Copy + AddAssign<T>,
    Vec<T>: Extend<U>,
{
    fn extend<E: IntoIterator<Item = U>>(&mut self, iter: E) {
        self.extend_by(iter, identity)
    }
}

impl<T: Copy> FenwickTree<T> {
    pub(crate) fn fix(&mut self, pos: usize)
    where
        T: AddAssign,
    {
        self.fix_by(pos, identity)
    }

    pub(crate) fn fix_by<B, F>(&mut self, pos: usize, mut f: F)
    where
        F: FnMut(T) -> B,
        T: AddAssign<B>,
    {
        for i in 1..self.tree.len() {
            let next = next_fwd(i);
            if pos <= next && next < self.tree.len() {
                let delta = f(self.tree[i]);
                self.tree[next] += delta;
            }
        }
    }

    fn from_slice<A: AsRef<[T]>>(slice: A, ident: T) -> Self
    where
        T: AddAssign,
    {
        let slice = slice.as_ref();

        let mut tree = vec![ident; slice.len() + 1];
        tree[1..].copy_from_slice(slice);

        let mut this = FenwickTree { tree };
        this.fix(0);
        this
    }

    pub(crate) fn extend_by<E, A, B, F>(&mut self, iter: E, f: F)
    where
        E: IntoIterator<Item = A>,
        Vec<T>: Extend<A>,
        F: FnMut(T) -> B,
        T: AddAssign<B>,
    {
        let bef = self.tree.len();
        self.tree.extend(iter);
        self.fix_by(bef, f);
    }

    pub(crate) fn extend_by_default<U, F>(&mut self, len: usize, f: F)
    where
        F: FnMut(T) -> U,
        T: Default + AddAssign<U>,
    {
        self.extend_by(std::iter::repeat(T::default()).take(len), f)
    }
}

fn prev_power_of_two(mut n: usize) -> usize {
    // if n == 0 {
    //     return n;
    // };
    if n > 0 {
        while n & (n - 1) > 0 {
            n = n & (n - 1);
        }
    }
    n
}

#[cfg(test)]
mod fenwick {
    use super::*;
    use quickcheck::quickcheck;

    type Fenwick<T> = FenwickTree<T>;

    quickcheck! {
        fn init(vec: Vec<usize>) -> bool {
            Fenwick::from_slice(&vec[..], 0) == {
                let mut bit = Fenwick::with_default(vec.len());
                for (i, &d) in vec.iter().enumerate() {
                    bit.add(i, d);
                }
                bit
            }
        }

        fn sum(vec: Vec<usize>) -> bool {
            let bit = Fenwick::from_slice(&vec, 0);
            (0..vec.len()).all(|i| {
                bit.sum::<usize>(i) == vec[..i].iter().sum::<usize>()
            })
        }

        // fn index_and_remain(vec: Vec<usize>, sums: Vec<usize>) -> bool {
        //     let bit = Fenwick::from_slice(&vec, 0);
        //     // dbg!(bit.sum::<usize>(vec.len()));

        //     sums.iter().scan(0, |a, sum| { *a += sum; Some(*a) }).take(vec.len()).all(|sum| {
        //         if let Ok((i, r)) = bit.search(sum) {
        //             bit.sum::<usize>(i) + r == sum
        //         } else {
        //             true
        //         }
        //     })
        // }
    }

    #[test]
    fn test_links() {
        let mut i = 3usize;
        while i < 512 {
            dbg!(i, format!("{:032b}", i));
            i = next_fwd(i);
        }

        let mut i = 7usize;
        while i > 0 {
            dbg!(i, format!("{:032b}", i));
            i = next_bwd(i);
        }
    }

    #[test]
    fn search2() {
        let mut bit = FenwickTree::from_slice(&vec![0u32], 0);

        // [0]
        assert_eq!(bit.search(0), Ok(0));
        assert_eq!(bit.search(1), Err(1));
        assert_eq!(bit.search(2), Err(1));

        // [0, 1, 2]
        bit.extend(vec![1, 2]);
        assert_eq!(bit.search(0), Ok(0));
        assert_eq!(bit.search(1), Ok(1));
        assert_eq!(bit.search(2), Ok(2));
        assert_eq!(bit.search(3), Ok(2));
        assert_eq!(bit.search(4), Err(3));

        // [0, 1, 2, 3, 4, 5, 6]
        bit.extend(vec![3, 4, 5, 6]);
        assert_eq!(bit.search(0), Ok(0));
        assert_eq!(bit.search(1), Ok(1));
        assert_eq!(bit.search(2), Ok(2));
        assert_eq!(bit.search(3), Ok(2));
        assert_eq!(bit.search(4), Ok(3));
        assert_eq!(bit.search(5), Ok(3));
        assert_eq!(bit.search(6), Ok(3));
        assert_eq!(bit.search(20), Ok(6));
        assert_eq!(bit.search(21), Ok(6));
        assert_eq!(bit.search(22), Err(7));
    }
}

use std::{fmt::Debug, iter::Peekable, slice, vec};

use crate::{
    bits::Word,
    bits::{Difference, Intersection, SymmetricDifference, Union},
    num::try_cast,
    ops::*,
};

use super::{Block, Loc1, Ordering, EQ, GT, LT};

// /// /// A 0 based sorted bit sequence.
// #[derive(Debug, Clone, Default, PartialEq, Eq)]
// pub(crate) struct Pos0(Vec<u16>);

macro_rules! impl_ops {
    ( $( $Loc:ident ),* ) => ($(
        // impl<P: Word> Deref for $Loc<P> {
        //     type Target = [P];
        //     #[inline]
        //     fn deref(&self) -> &Self::Target {
        //         &self.0
        //     }
        // }

        // impl<P: Word> Borrow<[P]> for $Loc<P> {
        //     #[inline]
        //     fn borrow(&self) -> &[P] {
        //         &self.0
        //     }
        // }

        // impl<P: Word> AsRef<[P]> for $Loc<P> {
        //     #[inline]
        //     fn as_ref(&self) -> &[P] {
        //         &self.0
        //     }
        // }

        impl<'a> IntoIterator for &'a $Loc {
            type Item = &'a u16;
            type IntoIter = slice::Iter<'a, u16>;
            fn into_iter(self) -> Self::IntoIter {
                self.data.iter()
            }
        }

        impl IntoIterator for $Loc {
            type Item = u16;
            type IntoIter = vec::IntoIter<u16>;
            fn into_iter(self) -> Self::IntoIter {
                self.data.into_iter()
            }
        }

        impl $Loc {
            #[inline]
            pub fn new() -> Self {
                Self{data:Vec::new()}
            }
            #[inline]
            pub fn with_capacity(cap: usize) -> Self {
                Self{data:Vec::with_capacity(cap)}
            }
        }
    )*)
}
impl_ops!(Loc1);

impl Loc1 {
    pub fn runs<'a>(&'a self) -> impl Iterator<Item = (usize, usize)> + 'a {
        // `Iterator::scan` should be better?
        struct RunIter<'b, I: Iterator<Item = &'b u16>>(Peekable<I>);
        impl<'b, I> Iterator for RunIter<'b, I>
        where
            I: Iterator<Item = &'b u16>,
        {
            type Item = (usize, usize);
            fn next(&mut self) -> Option<Self::Item> {
                self.0.next().and_then(|&n| {
                    let mut m = n;
                    let n = try_cast(n);
                    while let Some(&peek) = self.0.peek() {
                        if m + 1 == *peek {
                            m = *self.0.next().unwrap();
                            continue;
                        } else {
                            break;
                        }
                    }
                    Some((n, try_cast(m)))
                })
            }
        }
        RunIter(self.data.iter().peekable())
    }
}

impl FixedBits for Loc1 {
    const SIZE: usize = Block::BITS;
    #[inline]
    fn none() -> Self {
        Loc1 { data: Vec::new() }
    }
}

impl Bits for Loc1 {
    #[inline]
    fn size(&self) -> usize {
        Self::SIZE
    }

    #[inline]
    fn count1(&self) -> usize {
        self.data.len()
    }

    #[inline]
    fn bit(&self, i: usize) -> bool {
        self.data.binary_search(&try_cast(i)).is_ok()
    }

    fn getn<W: Word>(&self, i: usize, len: usize) -> W {
        assert!(len <= W::BITS && i < self.size() && i + len <= self.size());
        if len == 0 {
            return W::NONE;
        }

        match self.data.binary_search(&try_cast(i)) {
            Ok(loc) if loc + 1 < self.data.len() => {
                let mut out = W::I;
                for &b in self.data[loc + 1..]
                    .iter()
                    .take_while(|&x| try_cast::<u16, usize>(*x) < i + len)
                {
                    out.put1(try_cast::<u16, usize>(b) - i);
                }
                out
            }
            Err(loc) if loc < self.data.len() => {
                let mut out = W::O;
                for &b in self.data[loc..]
                    .iter()
                    .take_while(|&x| try_cast::<u16, usize>(*x) < i + len)
                {
                    out.put1(try_cast::<u16, usize>(b) - i);
                }
                out
            }

            // Ok(loc) if loc + 1 >= self.data.len() => W::I,
            // Err(loc) if loc >= self.data.len() => W::O,
            _ => unreachable!(),
        }
    }
}

impl BitsMut for Loc1 {
    fn put1(&mut self, i: usize) -> &mut Self {
        BOUNDS_CHECK!(i < self.size());
        let i = try_cast(i);
        if let Err(loc) = self.data.binary_search(&i) {
            self.data.insert(loc, i);
        }
        self
    }

    fn put0(&mut self, i: usize) -> &mut Self {
        BOUNDS_CHECK!(i < self.size());
        let i = try_cast(i);
        if let Ok(loc) = self.data.binary_search(&i) {
            self.data.remove(loc);
        }
        self
    }

    fn flip(&mut self, i: usize) -> &mut Self {
        BOUNDS_CHECK!(i < self.size());
        if self.bit(i) {
            self.put0(i)
        } else {
            self.put1(i)
        }
    }
}

impl BitRank for Loc1 {
    fn rank1(&self, i: usize, j: usize) -> usize {
        let rank = |p| {
            // Search the smallest index `p` that satisfy `vec[p] >= i`,
            // `k` also implies the number of enabled bits in [0, p).
            // For example, searching 5 in `[0, 1, 7]` return 2.
            match self.data.binary_search(&try_cast::<usize, u16>(p)) {
                Ok(p) | Err(p) => p,
            }
        };

        let cap = self.size();
        BOUNDS_CHECK!(i <= j && j <= cap);
        match (i, j) {
            (i, j) if i == j => 0,
            (0, i) if i == cap => self.count1(),
            (0, i) => rank(i),
            (i, j) if j == cap => self.count1() - rank(i),
            (i, j) => rank(j) - rank(i),
        }
    }
}

impl BitSelect for Loc1 {
    #[inline]
    fn select1(&self, n: usize) -> Option<usize> {
        self.data.get(n).map(|&x| try_cast(x))
    }
}

impl Intersection<Self> for Loc1 {
    fn intersection(&mut self, that: &Self) {
        self.data = Cmp {
            a: self.data.iter().peekable(),
            b: that.data.iter().peekable(),
        }
        .cloned()
        .collect();

        struct Cmp<L: Iterator, R: Iterator> {
            a: Peekable<L>,
            b: Peekable<R>,
        }
        impl<L, R, T: Ord + Debug> Iterator for Cmp<L, R>
        where
            L: Iterator<Item = T>,
            R: Iterator<Item = T>,
        {
            type Item = T;
            fn next(&mut self) -> Option<Self::Item> {
                loop {
                    match Ord::cmp(self.a.peek()?, self.b.peek()?) {
                        LT => {
                            self.a.next();
                        }
                        EQ => {
                            let a = self.a.next().unwrap();
                            let b = self.b.next().unwrap();
                            assert_eq!(a, b);
                            return Some(a);
                        }
                        GT => {
                            self.b.next();
                        }
                    }
                }
            }
        }
    }
}

#[inline]
fn cmp_opt<T: Ord>(x: Option<&T>, y: Option<&T>, a: Ordering, b: Ordering) -> Ordering {
    match (x, y) {
        (None, _) => a,
        (_, None) => b,
        (Some(x), Some(y)) => x.cmp(y),
    }
}

impl Union<Self> for Loc1 {
    fn union(&mut self, that: &Self) {
        self.data = Cmp {
            a: self.data.iter().peekable(),
            b: that.data.iter().peekable(),
        }
        .cloned()
        .collect();

        struct Cmp<L: Iterator, R: Iterator> {
            a: Peekable<L>,
            b: Peekable<R>,
        }
        impl<L, R, T: Ord + Debug> Iterator for Cmp<L, R>
        where
            L: Iterator<Item = T>,
            R: Iterator<Item = T>,
        {
            type Item = T;
            fn next(&mut self) -> Option<Self::Item> {
                match cmp_opt(self.a.peek(), self.b.peek(), GT, LT) {
                    LT => self.a.next(),
                    EQ => {
                        let a = self.a.next().unwrap();
                        let b = self.b.next().unwrap();
                        assert_eq!(a, b);
                        Some(a)
                    }
                    GT => self.b.next(),
                }
            }
        }
    }
}

impl Difference<Self> for Loc1 {
    fn difference(&mut self, that: &Self) {
        self.data = Cmp {
            a: self.data.iter().peekable(),
            b: that.data.iter().peekable(),
        }
        .cloned()
        .collect();

        struct Cmp<L: Iterator, R: Iterator> {
            a: Peekable<L>,
            b: Peekable<R>,
        }
        impl<L, R, T: Ord + Debug> Iterator for Cmp<L, R>
        where
            L: Iterator<Item = T>,
            R: Iterator<Item = T>,
        {
            type Item = T;
            fn next(&mut self) -> Option<Self::Item> {
                loop {
                    match cmp_opt(self.a.peek(), self.b.peek(), LT, LT) {
                        LT => return self.a.next(),
                        EQ => {
                            let a = self.a.next().unwrap();
                            let b = self.b.next().unwrap();
                            assert_eq!(a, b);
                            // return Some(a);
                        }
                        GT => {
                            self.b.next();
                        }
                    }
                }
            }
        }
    }
}

impl SymmetricDifference<Self> for Loc1 {
    fn symmetric_difference(&mut self, that: &Self) {
        self.data = Cmp {
            a: self.data.iter().peekable(),
            b: that.data.iter().peekable(),
        }
        .cloned()
        .collect();

        struct Cmp<L: Iterator, R: Iterator> {
            a: Peekable<L>,
            b: Peekable<R>,
        }
        impl<L, R, T: Ord + Debug> Iterator for Cmp<L, R>
        where
            L: Iterator<Item = T>,
            R: Iterator<Item = T>,
        {
            type Item = T;
            fn next(&mut self) -> Option<Self::Item> {
                loop {
                    match cmp_opt(self.a.peek(), self.b.peek(), GT, LT) {
                        LT => return self.a.next(),
                        EQ => {
                            let a = self.a.next().unwrap();
                            let b = self.b.next().unwrap();
                            assert_eq!(a, b);
                            // return Some(Member::And { lhs, rhs });
                        }
                        GT => return self.b.next(),
                    }
                }
            }
        }
    }
}

// impl<P: Word> mask::Intersection<Self> for Loc1<P> {
//     fn intersection(&mut self, that: &Self) {
//         let mut n = 0;
//         let mut i = 0;
//         let mut j = 0;
//         while i < self.len() && j < that.len() {
//             match self[i].cmp(&that[j]) {
//                 LT => i += 1,
//                 EQ => {
//                     self.data.swap(n, i); // self[n] = self[i]
//                     n += 1;
//                     i += 1;
//                     j += 1;
//                 }
//                 GT => j += 1,
//             }
//         }
//         self.data.truncate(n);
//     }
// }

// impl<P: Word> mask::Union<Self> for Loc1<P> {
//     fn union(&mut self, that: &Self) {
//         let mut i = 0;
//         let mut pos1 = that.iter();
//         'RHS: for &b in &mut pos1 {
//             while i < self.len() {
//                 match self[i].cmp(&b) {
//                     LT => i += 1,
//                     EQ => continue 'RHS,
//                     GT => self.data.insert(i, b),
//                 }
//             }
//             self.data.push(b);
//             break;
//         }
//         self.data.extend(pos1);
//     }
// }

// impl<P: Word> mask::Difference<Self> for Loc1<P> {
//     fn difference(&mut self, that: &Self) {
//         let mut i = 0;
//         let mut pos1 = that.iter();
//         let mut curr = pos1.next();
//         while i < self.len() {
//             match curr.map(|b| self[i].cmp(b)) {
//                 Some(LT) => {
//                     i += 1;
//                 }
//                 Some(EQ) => {
//                     self.data.remove(i);
//                     curr = pos1.next();
//                 }
//                 Some(GT) => {
//                     curr = pos1.next();
//                 }
//                 None => break,
//             }
//         }
//     }
// }

// impl<P: Word> mask::SymmetricDifference<Self> for Loc1<P> {
//     fn symmetric_difference(&mut self, that: &Self) {
//         let mut i = 0;
//         let mut pos1 = that.iter();
//         let mut curr = pos1.next();
//         while i < self.len() {
//             match curr.map(|c| self[i].cmp(c)) {
//                 Some(LT) => {
//                     i += 1;
//                 }
//                 Some(EQ) => {
//                     self.data.remove(i);
//                     curr = pos1.next();
//                 }
//                 Some(GT) => {
//                     self.data.insert(i, *curr.unwrap());
//                     curr = pos1.next();
//                     i += 1;
//                 }
//                 None => break,
//             }
//         }
//         if let Some(&c) = curr {
//             self.data.push(c);
//             self.data.extend(pos1);
//         }
//     }
// }

use std::cmp;

// pub struct Merge<L, R>
// where
//     L: Iterator,
//     R: Iterator,
// {
//     lhs: std::iter::Peekable<L>,
//     rhs: std::iter::Peekable<R>,
// }

// impl<L, R> Merge<L, R>
// where
//     L: Iterator,
//     R: Iterator,
// {
//     pub fn merge<A, B, T>(lhs: A, rhs: B) -> Self
//     where
//         A: IntoIterator<Item = T, IntoIter = L>,
//         B: IntoIterator<Item = T, IntoIter = R>,
//         L: Iterator<Item = T>,
//         R: Iterator<Item = T>,
//         T: Ord,
//     {
//         let lhs = lhs.into_iter().peekable();
//         let rhs = rhs.into_iter().peekable();
//         Merge { lhs, rhs }
//     }
// }

// impl<L, R, T> Iterator for Merge<L, R>
// where
//     L: Iterator<Item = T>,
//     R: Iterator<Item = T>,
//     T: Ord,
// {
//     type Item = T;
//     fn next(&mut self) -> Option<Self::Item> {
//         match (self.lhs.peek(), self.rhs.peek()) {
//             (None, _) => self.rhs.next(),
//             (_, None) => self.lhs.next(),
//             (Some(lhs), Some(rhs)) => match lhs.cmp(rhs) {
//                 cmp::Ordering::Less | cmp::Ordering::Equal => self.lhs.next(),
//                 cmp::Ordering::Greater => self.rhs.next(),
//             },
//         }
//     }
// }

pub struct MergeBy<L, R, F>
where
    L: Iterator,
    R: Iterator,
    F: Fn(&L::Item, &R::Item) -> std::cmp::Ordering,
{
    lhs: std::iter::Peekable<L>,
    rhs: std::iter::Peekable<R>,
    fun: F,
}

impl<L, R, F> MergeBy<L, R, F>
where
    L: Iterator,
    R: Iterator,
    F: Fn(&L::Item, &R::Item) -> std::cmp::Ordering,
{
    pub fn merge_by<A, B, T>(lhs: A, rhs: B, fun: F) -> Self
    where
        A: IntoIterator<Item = T, IntoIter = L>,
        B: IntoIterator<Item = T, IntoIter = R>,
        L: Iterator<Item = T>,
        R: Iterator<Item = T>,
        F: Fn(&T, &T) -> std::cmp::Ordering,
    {
        let lhs = lhs.into_iter().peekable();
        let rhs = rhs.into_iter().peekable();
        MergeBy { lhs, rhs, fun }
    }
}

impl<L, R, F, T> Iterator for MergeBy<L, R, F>
where
    L: Iterator<Item = T>,
    R: Iterator<Item = T>,
    F: Fn(&T, &T) -> std::cmp::Ordering,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        match (self.lhs.peek(), self.rhs.peek()) {
            (None, _) => self.rhs.next(),
            (_, None) => self.lhs.next(),
            (Some(lhs), Some(rhs)) => match (self.fun)(lhs, rhs) {
                cmp::Ordering::Less | cmp::Ordering::Equal => self.lhs.next(),
                cmp::Ordering::Greater => self.rhs.next(),
            },
        }
    }
}

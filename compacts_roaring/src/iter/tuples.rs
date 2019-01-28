pub struct Tuples<I: Iterator> {
    iter: I,
    last: Option<I::Item>,
}

impl<I: Iterator> Tuples<I> {
    pub fn tuples(mut iter: I) -> Self {
        let last = iter.next();
        Tuples { iter, last }
    }
}

impl<I> Iterator for Tuples<I>
where
    I: Iterator,
    I::Item: Clone,
{
    type Item = (I::Item, I::Item);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(last) = self.last.clone() {
            self.last = self.iter.next();
            if let Some(next) = self.last.clone() {
                return Some((last, next));
            }
        }
        None
    }
}

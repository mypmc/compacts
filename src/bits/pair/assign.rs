pub trait Assign<T = Self> {
    fn and_assign(&mut self, that: T);
    fn or_assign(&mut self, that: T);
    fn and_not_assign(&mut self, that: T);
    fn xor_assign(&mut self, that: T);
}

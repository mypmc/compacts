use std::cell::RefCell;

#[derive(Debug, Clone)]
struct UnionFind<T> {
    cell: RefCell<Vec<T>>,
}

macro_rules! impl_UnionFind {
    ($( $ty:ty ),*) => ($(
        impl UnionFind<$ty> {
            fn new(size: usize) -> Self {
                UnionFind {
                    cell: RefCell::new(vec![0; size]),
                }
            }

            fn root(&self, i: $ty) -> $ty {
                assert_ne!(i, 0);

                let mut data = self.cell.borrow_mut();
                let mut root = i;
                while data[root as usize] != 0 {
                    root = data[root as usize];
                };
                if i != root {
                    data[i as usize] = root;
                }
                root
            }

            fn same(&self, i: $ty, j: $ty) -> bool {
                self.root(i) == self.root(j)
            }

            fn join(&mut self, i: $ty, j: $ty) -> bool {
                let i = self.root(i);
                let j = self.root(j);
                if i != j {
                    self.cell.borrow_mut()[j as usize] = i;
                }
                i != j
            }
        }
    )*)
}
impl_UnionFind!(u8, u16, u32, u64);

#[test]
fn union_find() {
    macro_rules! check_UnionFind {
        ($( $ty:ty ),*) => ($(
            let mut uf = UnionFind::<$ty>::new(100);
            for i in 1..99 {
                assert!(uf.join(i + 1, 1));
            }
            for i in 1..99 {
                assert!(uf.same(1, i + 1));
            }
        )*)
    }

    check_UnionFind!(u8, u16, u32, u64);
}

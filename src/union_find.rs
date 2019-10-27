#![allow(dead_code)]

use std::cell::RefCell;

#[derive(Debug, Clone)]
struct UnionFind {
    cell: RefCell<Vec<usize>>,
}

impl UnionFind {
    fn new(size: usize) -> Self {
        UnionFind {
            cell: RefCell::new(vec![0; size]),
        }
    }

    fn root(&self, i: usize) -> usize {
        assert_ne!(i, 0);
        let mut data = self.cell.borrow_mut();
        let mut root = i;
        while data[root] != 0 {
            root = data[root];
        }
        if i != root {
            data[i] = root;
        }
        root
    }

    fn same(&self, i: usize, j: usize) -> bool {
        self.root(i) == self.root(j)
    }

    fn join(&mut self, i: usize, j: usize) -> bool {
        let i = self.root(i);
        let j = self.root(j);
        if i != j {
            self.cell.borrow_mut()[j] = i;
        }
        i != j
    }
}

#[test]
fn union_find() {
    let mut uf = UnionFind::new(100);

    uf.join(1, 9);
    uf.join(3, 9);
    uf.join(5, 9);
    uf.join(7, 9);

    uf.join(2, 10);
    uf.join(4, 10);
    uf.join(6, 4);
    uf.join(8, 2);

    assert!(uf.same(1, 9));
    assert!(uf.same(3, 9));
    assert!(uf.same(5, 9));
    assert!(uf.same(7, 9));
    assert!(uf.same(9, 9));

    assert!(uf.same(2, 4));
    assert!(uf.same(4, 6));
    assert!(uf.same(6, 8));
    assert!(uf.same(8, 10));
}

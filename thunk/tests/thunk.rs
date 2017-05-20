#![allow(non_snake_case)]

#[macro_use]
extern crate thunk;
use thunk::Thunk;

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::thread;

#[test]
fn thunks_in_Vec() {
    let arc = Arc::new(Mutex::new(Vec::new()));
    let mut vec = vec![lazy!({
                                 arc.lock().unwrap().push(0);
                                 0
                             }),
                       lazy!({
                                 arc.lock().unwrap().push(1);
                                 1
                             }),
                       lazy!({
                                 arc.lock().unwrap().push(2);
                                 2
                             }),
                       eval!(3)];

    assert_eq!(vec.len(), 4);
    let removed = vec.remove(2);
    assert_eq!(vec.len(), 3);

    for thunk in &vec {
        Thunk::force(thunk); // can't unwrap because unwrap need owenership.
    }

    {
        let locked = arc.lock().unwrap();
        assert_eq!(*locked, vec![0, 1], "{:?}", *locked);
    }
    assert_eq!(Thunk::unwrap(removed), 2); // removed thunk evaluate here.
    {
        let locked = arc.lock().unwrap();
        assert_eq!(*locked, vec![0, 1, 2], "{:?}", *locked);
    }
}

#[test]
fn thunks_in_BTreeMap() {
    let arc = Arc::new(Mutex::new(Vec::new()));
    let mut map1 = BTreeMap::new();
    let mut map2 = BTreeMap::new();
    for i in 0..10 {
        {
            let arc1 = arc.clone();
            let j1 = i;
            map1.insert(i,
                        lazy!(move {
                              arc1.lock().unwrap().push(10 - j1);
                              10 - j1
                          }));
        }
        {
            let arc2 = arc.clone();
            let j2 = i;
            map2.insert(i,
                        lazy!(move {
                              arc2.lock().unwrap().push(j2);
                              j2
                          }));
        }
    }

    assert_eq!(map1.len(), 10);
    assert_eq!(map2.len(), 10);

    for i in 0..10 {
        if i % 2 == 0 {
            let c1 = map1[&i].clone(); // evaluate thunk here,
            let c2 = map2[&i].clone(); // and here.
            map1.insert(i, lazy!(move {*c1 + *c2}));
        } else {
            map1.remove(&i);
            map2.remove(&i);
        }
    }

    for &key in map1.keys() {
        assert_eq!(*map1[&key], 10, "{:?} {:?}", key, *map1[&key]);
    }

    let v = arc.lock().unwrap();
    assert_eq!(*v, vec![10, 0, 8, 2, 6, 4, 4, 6, 2, 8], "{:?}", *v);
}

#[test]
fn print_once() {
    let expr = lazy!({
                         println!("evaluated!");
                         7
                     });

    assert_eq!(*expr, 7); // "evaluated!" printed here.
    assert_eq!(*expr, 7); // Nothing printed.
    assert_eq!(Thunk::unwrap(expr), 7);
}

#[test]
fn evaluate_at_deref() {
    let value = lazy!(1000);
    assert_eq!(*value, 1000);
}

#[test]
fn evaluate_just_once() {
    let c1 = Arc::new(Mutex::new(0));
    let c2 = c1.clone();

    let value = lazy!({
                          let mut data = c1.lock().unwrap();
                          *data += 1;
                      });

    assert_eq!(*c2.lock().unwrap(), 0);
    Thunk::force(&value);
    assert_eq!(*c2.lock().unwrap(), 1);
    Thunk::force(&value);
    assert_eq!(*c2.lock().unwrap(), 1);
}

pub struct DropTest(Arc<Mutex<u64>>);
impl DropTest {
    fn value(&self) -> u64 {
        let DropTest(ref c) = *self;
        *c.lock().unwrap()
    }
}
impl Drop for DropTest {
    fn drop(&mut self) {
        let DropTest(ref c) = *self;
        *c.lock().unwrap() += 1;
        println!("Dropped {:?} ", c);
    }
}

#[test]
fn drop_just_once() {
    let c1 = Arc::new(Mutex::new(0));
    let c2 = c1.clone();

    let th = thread::spawn(move || {
        let drop = DropTest(c2);
        let lazy = lazy!(move {
            let drop_ref = &drop;
            assert_eq!(drop_ref.value(), 0, "drop_ref:{:?}", drop_ref.value());
            // DropTest::drop invoke here
        });
        Thunk::force(&lazy);
    });

    match th.join() {
        Ok(_) => assert_eq!(*c1.lock().unwrap(), 1),
        Err(_) => unreachable!(),
    }
}

#[test]
fn thunk_in_thunk() {
    let t1 = lazy!({
                       println!("t1");
                       1 + 2
                   });
    let t2 = lazy!({
                       println!("t2");
                       3 + 4
                   });

    let t3 = lazy!(move {
        println!("evaluate thunk in thunk");
        let r1 = *t1;
        let r2 = *t2;
        (r1 + r2) * r1
    });

    assert_eq!(*t3, 30);
}

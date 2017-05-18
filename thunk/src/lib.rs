#![feature(fnbox)]

use std::fmt::{self, Debug, Formatter};
use std::ops::{Deref, DerefMut};

use std::boxed::FnBox;
use std::cell::UnsafeCell;
use std::ptr;

pub struct Thunk<'a, T> {
    cell: UnsafeCell<Expr<'a, T>>,
}

#[macro_export]
macro_rules! lazy {
    ( move $e:expr ) => {$crate::Thunk::lazy(move || { $e })};
    ( $e:expr )      => {$crate::Thunk::lazy(|| { $e })};
}

#[macro_export]
macro_rules! eval {
    ( $e:expr ) => {$crate::Thunk::eval($e)};
}

impl<'a, T> Thunk<'a, T> {
    /// Create a new thunk. This thunk will be evaluated lazily.
    pub fn lazy<F>(f: F) -> Thunk<'a, T>
        where F: 'a + FnOnce() -> T
    {
        let expr = Expr::Lazy(Yield::new(f));
        let cell = UnsafeCell::new(expr);
        Thunk { cell }
    }

    /// Create a new, evaluated, thunk.
    pub fn eval<'b>(value: T) -> Thunk<'b, T> {
        Thunk { cell: UnsafeCell::new(Expr::Eval(value)) }
    }

    /// Evaluate a thunk.
    pub fn force(thunk: &Self) {
        unsafe {
            match *thunk.cell.get() {
                Expr::Lazy(_) => (),
                Expr::InProgress => panic!("Thunk::force called recursively"),
                Expr::Eval(_) => return, // already forced
            };
            match ptr::replace(thunk.cell.get(), Expr::InProgress) {
                Expr::Lazy(f) => *thunk.cell.get() = Expr::Eval(f.invoke()),
                _ => unreachable!(),
            };
        }
    }

    /// Return the value and consume the thunk.
    pub fn unwrap(thunk: Self) -> T {
        Self::force(&thunk);
        match unsafe { thunk.cell.into_inner() } {
            Expr::Eval(v) => v,
            _ => unreachable!(),
        }
    }
}

enum Expr<'a, T> {
    Lazy(Yield<'a, T>),
    InProgress,
    Eval(T),
}

struct Yield<'a, T> {
    boxed: Box<FnBox() -> T + 'a>,
}

impl<'a, T> Yield<'a, T> {
    fn new<F: 'a + FnOnce() -> T>(f: F) -> Yield<'a, T> {
        let boxed = Box::new(f);
        Yield { boxed }
    }

    fn invoke(self) -> T {
        (self.boxed)()
    }
}

impl<'a, T> Deref for Thunk<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        Thunk::force(self);
        match *unsafe { &*self.cell.get() } {
            Expr::Eval(ref val) => val,
            _ => unreachable!("Thunk::deref failed"),
        }
    }
}

impl<'a, T> DerefMut for Thunk<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        Thunk::force(self);
        match *unsafe { &mut *self.cell.get() } {
            Expr::Eval(ref mut val) => val,
            _ => unreachable!("Thunk::deref_mut failed"),
        }
    }
}

impl<'a, T> Clone for Thunk<'a, T>
    where T: Clone
{
    fn clone(&self) -> Self {
        Thunk::eval((**self).clone())
    }
}

impl<'a, T> Debug for Thunk<'a, T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.pad("Thunk")
    }
}

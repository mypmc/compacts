use std::boxed::FnBox;
use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut, Drop};
use std::sync::atomic::{AtomicU8, Ordering};

use parking_lot::{Mutex, MutexGuard};

struct State(u8);

const INIT: State = State(0);
const WAIT: State = State(1);
const FREE: State = State(2);

macro_rules! lazy {
    ( $e:expr ) => { $crate::bits::thunk::Thunk::lazy(move || { $e }) };
}
macro_rules! eval {
    ( $e:expr ) => { $crate::bits::thunk::Thunk::eval($e) };
}

impl Deref for State {
    type Target = u8;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Default)]
pub struct Lock<T> {
    atom: AtomicU8,
    lock: Mutex<()>,
    cell: UnsafeCell<T>,
}
unsafe impl<T: Send> Send for Lock<T> {}
unsafe impl<T: Sync> Sync for Lock<T> {}

pub struct Thunk<'a, T> {
    once: Lock<Expr<'a, T>>,
}
unsafe impl<'a, T: Send> Send for Thunk<'a, T> {}
unsafe impl<'a, T: Sync> Sync for Thunk<'a, T> {}

enum Expr<'a, T> {
    Deferred(Yield<'a, T>),
    InProgress,
    Evaluated(T),
}
unsafe impl<'a, T: Send> Send for Expr<'a, T> {}
unsafe impl<'a, T: Sync> Sync for Expr<'a, T> {}

struct Yield<'a, T> {
    boxed: Box<FnBox() -> T + 'a>,
}

impl<T> Lock<T>
where
    T: Send + Sync,
{
    pub fn new(inner: T) -> Lock<T> {
        Lock {
            atom: AtomicU8::new(*INIT),
            lock: Mutex::new(()),
            cell: UnsafeCell::new(inner),
        }
    }

    pub fn try_lock(&self) -> Option<LockGuard<T>> {
        // Ordering::Acquire
        if self.atom.compare_and_swap(*INIT, *WAIT, Ordering::SeqCst) == *INIT {
            Some(LockGuard::new(self))
        } else {
            None
        }
    }

    fn is_locked(&self) -> bool {
        // Ordering::Release
        self.atom.load(Ordering::SeqCst) == *WAIT
    }

    pub fn wait(&self) {
        if self.is_locked() {
            let _ = self.lock.lock();
        }
    }

    // pub fn into_inner(self) -> T {
    //     // this is safe, because of `self`.
    //     unsafe { self.cell.into_inner() }
    // }
}

impl<T> Deref for Lock<T>
where
    T: Send + Sync,
{
    type Target = T;

    /// Dereference to the value inside the Lock.
    /// This can block if the Lock is in its lock state.
    fn deref(&self) -> &T {
        if self.atom.compare_and_swap(*INIT, *FREE, Ordering::SeqCst) == *WAIT {
            self.wait();
        }
        debug_assert_eq!(self.atom.load(Ordering::SeqCst), *FREE);
        unsafe { &*self.cell.get() }
    }
}

impl<T> DerefMut for Lock<T>
where
    T: Send + Sync,
{
    fn deref_mut(&mut self) -> &mut T {
        // `&mut self` means no LockGuard's exist.
        debug_assert_ne!(self.atom.load(Ordering::SeqCst), *WAIT);
        unsafe { &mut *self.cell.get() }
    }
}

pub struct LockGuard<'a, T: 'a> {
    mutex: &'a Lock<T>,
    _guard: MutexGuard<'a, ()>,
}

impl<'a, T> LockGuard<'a, T>
where
    T: 'a,
{
    fn new(mutex: &'a Lock<T>) -> LockGuard<'a, T> {
        let _guard = mutex.lock.lock();
        LockGuard { mutex, _guard }
    }
}

impl<'a, T> ::std::fmt::Debug for LockGuard<'a, T>
where
    T: Send + Sync,
{
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        f.pad("LockGuard")
    }
}

impl<'a, T> Deref for LockGuard<'a, T>
where
    T: Send + Sync,
{
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.mutex.cell.get() }
    }
}
impl<'a, T> DerefMut for LockGuard<'a, T>
where
    T: Send + Sync,
{
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.cell.get() }
    }
}

impl<'a, T> Drop for LockGuard<'a, T> {
    fn drop(&mut self) {
        self.mutex.atom.store(*FREE, Ordering::SeqCst);
    }
}

impl<'a, T: Send + Sync> Thunk<'a, T> {
    pub fn lazy<F>(f: F) -> Self
    where
        F: 'a + FnBox() -> T + Send + Sync,
    {
        let expr = Expr::Deferred(Yield::new(f));
        let once = Lock::new(expr);
        Self { once }
    }

    pub fn eval(val: T) -> Thunk<'a, T> {
        let once = Lock::new(Expr::Evaluated(val));
        once.try_lock();
        Thunk { once }
    }

    pub fn force(thunk: &Self) {
        match thunk.once.try_lock() {
            Some(mut lock) => match ::std::mem::replace(&mut *lock, Expr::InProgress) {
                Expr::Deferred(f) => *lock = Expr::Evaluated(f.invoke()),
                _ => unreachable!("Lock locked, but an inner expr is not defferred."),
            },
            None => thunk.once.wait(),
        }
    }
}

impl<'a, T: Send + Sync> Deref for Thunk<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        Self::force(self);
        match *self.once {
            Expr::Evaluated(ref val) => val,
            _ => unreachable!("invoked force, but hold unevaluated value"),
        }
    }
}
impl<'a, T: Send + Sync> DerefMut for Thunk<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        Self::force(self);
        match *self.once {
            Expr::Evaluated(ref mut val) => unsafe { ::std::mem::transmute(val) },
            _ => unreachable!("invoked force, but hold unevaluated value"),
        }
    }
}

impl<'a, T> Yield<'a, T> {
    fn new<F>(f: F) -> Yield<'a, T>
    where
        F: 'a + FnBox() -> T,
    {
        let boxed = Box::new(f);
        Yield { boxed }
    }

    fn invoke(self) -> T {
        (self.boxed)()
    }
}

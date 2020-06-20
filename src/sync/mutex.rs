use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{self, AtomicBool, Ordering},
};

#[derive(Debug)]
pub struct Mutex<T: ?Sized> {
    lock: AtomicBool,
    val: UnsafeCell<T>,
}
unsafe impl<T: Send> Send for Mutex<T> {}
unsafe impl<T: Sync> Sync for Mutex<T> {}

impl<T> Mutex<T> {
    pub const fn new(val: T) -> Self {
        Self {
            lock: AtomicBool::new(false),
            val: UnsafeCell::new(val),
        }
    }
}

impl<'a, T: ?Sized> Mutex<T> {
    pub fn lock(&'a self) -> MutexGuard<'a, T> {
        while self.lock.compare_and_swap(false, true, Ordering::Acquire) {
            atomic::spin_loop_hint();
        }

        MutexGuard {
            lock: &self.lock,
            val: unsafe { &mut *self.val.get() },
        }
    }
}

pub struct MutexGuard<'a, T: ?Sized> {
    lock: &'a AtomicBool,
    val: &'a mut T,
}

impl<T: ?Sized> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.store(false, Ordering::Release);
    }
}

impl<'a, T: ?Sized> Deref for MutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.val
    }
}
impl<'a, T: ?Sized> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.val
    }
}

use core::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    ops::Deref,
    sync::atomic::{self, AtomicUsize, Ordering},
};

const UNINIT: usize = 0x0;
const RUNNING: usize = 0x1;
const INIT: usize = 0x2;

#[derive(Debug)]
pub struct Once<T> {
    state: AtomicUsize,
    val: UnsafeCell<MaybeUninit<T>>,
}
unsafe impl<T: Send> Send for Once<T> {}
unsafe impl<T: Send + Sync> Sync for Once<T> {}

impl<T> Once<T> {
    pub const fn new() -> Self {
        Self {
            state: AtomicUsize::new(UNINIT),
            val: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    pub fn init_once<F: FnOnce() -> T>(&self, f: F) -> &T {
        let mut state = self.state.load(Ordering::SeqCst);

        if state == UNINIT {
            state = self
                .state
                .compare_and_swap(UNINIT, RUNNING, Ordering::SeqCst);

            if state == UNINIT {
                unsafe { (*self.val.get()).as_mut_ptr().write(f()) }
                self.state.store(INIT, Ordering::SeqCst);

                return unsafe { self.force_get() };
            }
        }

        loop {
            match state {
                RUNNING => {
                    atomic::spin_loop_hint();
                    state = self.state.load(Ordering::SeqCst);
                }
                INIT => break unsafe { self.force_get() },
                _ => unreachable!(),
            }
        }
    }

    pub fn try_get(&self) -> Option<&T> {
        match self.state.load(Ordering::SeqCst) {
            INIT => Some(unsafe { self.force_get() }),
            RUNNING | UNINIT => None,
            _ => unreachable!(),
        }
    }

    /// # Safety
    /// Creates a reference to uninitialised memory if the instance isn't initialised
    pub unsafe fn force_get(&self) -> &T {
        &*(*self.val.get()).as_ptr()
    }
}

impl<T> Default for Once<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct Lazy<T, F = fn() -> T>
where
    F: FnOnce() -> T,
{
    once: Once<T>,
    init: UnsafeCell<F>,
}
unsafe impl<T, F> Send for Lazy<T, F>
where
    T: Send,
    F: FnOnce() -> T + Send,
{
}
unsafe impl<T, F> Sync for Lazy<T, F>
where
    T: Send + Sync,
    F: FnOnce() -> T + Send + Sync,
{
}

impl<T, F> Lazy<T, F>
where
    F: FnOnce() -> T,
{
    pub const fn new(init: F) -> Self {
        Self {
            once: Once::new(),
            init: UnsafeCell::new(init),
        }
    }
}

impl<T, F> Deref for Lazy<T, F>
where
    F: FnOnce() -> T,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.once.init_once(unsafe { self.init.get().read() })
    }
}

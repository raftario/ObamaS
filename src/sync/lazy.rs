use core::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    ops::Deref,
    sync::atomic::{self, AtomicUsize, Ordering},
};

const UNINIT: usize = 0x0;
const RUNNING: usize = 0x1;
const INIT: usize = 0x2;

pub struct Lazy<T> {
    state: AtomicUsize,
    val: UnsafeCell<MaybeUninit<T>>,
    init: fn() -> T,
}
unsafe impl<T: Send> Send for Lazy<T> {}
unsafe impl<T: Send + Sync> Sync for Lazy<T> {}

impl<T> Lazy<T> {
    pub const fn new(init: fn() -> T) -> Self {
        Self {
            state: AtomicUsize::new(UNINIT),
            val: UnsafeCell::new(MaybeUninit::uninit()),
            init,
        }
    }
}

impl<T> Deref for Lazy<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        let mut state = self.state.load(Ordering::SeqCst);

        if state == UNINIT {
            state = self
                .state
                .compare_and_swap(UNINIT, RUNNING, Ordering::SeqCst);

            if state == UNINIT {
                unsafe { (*self.val.get()).as_mut_ptr().write((self.init)()) }
                self.state.store(INIT, Ordering::SeqCst);

                return unsafe { &*(*self.val.get()).as_ptr() };
            }
        }

        loop {
            match state {
                RUNNING => {
                    atomic::spin_loop_hint();
                    state = self.state.load(Ordering::SeqCst);
                }
                INIT => break unsafe { &*(*self.val.get()).as_ptr() },
                _ => unreachable!(),
            }
        }
    }
}

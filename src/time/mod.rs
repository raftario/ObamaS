use core::sync::atomic::AtomicUsize;

pub static TICKS: AtomicUsize = AtomicUsize::new(0);

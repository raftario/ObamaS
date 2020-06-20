pub trait Test {
    fn run(&self);
}
impl<T: Fn()> Test for T {
    fn run(&self) {
        s1print!("{} ... ", core::any::type_name::<T>());
        self();
        s1println!("ok")
    }
}

pub fn runner(tests: &[&dyn Test]) {
    s1println!(
        "running {} {}",
        tests.len(),
        if tests.len() == 1 { "test" } else { "tests" },
    );
    for test in tests {
        test.run();
    }
    crate::qemu::exit(crate::qemu::ExitCode::Success)
}

pub fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    s1println!("err");
    s1println!("{}", info);
    crate::qemu::exit(crate::qemu::ExitCode::Failed);
    crate::halt();
}

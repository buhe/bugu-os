#![feature(asm)]
#![feature(global_asm)]
#![feature(alloc_error_handler)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![no_std]
#![no_main]

#![reexport_test_harness_main = "test_main"]

extern crate alloc;
#[macro_use]
mod console;
mod heap;
mod lang;
mod scall_sbi;
mod task;
mod trap;

global_asm!(include_str!("stack.asm"));
global_asm!(include_str!("link_app.S"));

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
}

#[no_mangle]
extern "C" fn rust_main() -> ! {
    #[cfg(test)]
    test_main();

    clear_bss();
    heap::init();
    heap::heap_test();
    trap::init();
    task::init();
    task::run();
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    println!("1111111Running {} tests", tests.len());
    for test in tests {
        test();
    }
}


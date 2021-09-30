#![feature(asm)]
#![feature(global_asm)]
#![no_std]
#![no_main]

#[macro_use]
mod console;
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
    clear_bss();
    trap::init();
    task::init();
    task::run();
}

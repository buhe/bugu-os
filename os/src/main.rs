#![feature(llvm_asm)]
#![feature(global_asm)]
#![no_std]
#![no_main]

use scall_sbi::shutdown;

mod lang;
mod scall_sbi;

global_asm!(include_str!("stack.asm"));

#[no_mangle]
extern "C" fn rust_main() ->! {
    shutdown();
}

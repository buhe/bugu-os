#[inline(always)]
fn scall(which: usize, arg0: usize, arg1: usize, arg2: usize) -> usize {
    let mut ret;
    unsafe {
        llvm_asm!("ecall"
            : "={x10}" (ret)
            : "{x10}" (arg0), "{x11}" (arg1), "{x12}" (arg2), "{x17}" (which)
            : "memory"
            : "volatile"
        );
    }
    ret
}

const SBI_SHUTDOWN: usize = 8;

pub fn shutdown() -> ! {
    scall(SBI_SHUTDOWN, 0, 0, 0);
    panic!("It should shutdown!");
}

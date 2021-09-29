#[inline(always)]
fn scall(which: usize, arg0: usize, arg1: usize, arg2: usize) -> usize {
    let mut ret;
    unsafe {
        asm!("ecall",
            in( "x10") arg0,
            in("x11") arg1,
            in ("x12") arg2,
            in("x17") which,
            lateout("x10") ret,
        );
    }
    ret
}
const SBI_CONSOLE_PUTCHAR: usize = 1;
const SBI_SHUTDOWN: usize = 8;

pub fn shutdown() -> ! {
    scall(SBI_SHUTDOWN, 0, 0, 0);
    panic!("It should shutdown!");
}

pub fn put_char(c: usize) {
    scall(SBI_CONSOLE_PUTCHAR, c, 0, 0);
}

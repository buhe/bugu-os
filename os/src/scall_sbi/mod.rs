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
            options(nostack)
        );
    }
    ret
}

const SBI_SET_TIMER: usize = 0;
const SBI_CONSOLE_PUTCHAR: usize = 1;
const SBI_SHUTDOWN: usize = 8;

pub fn shutdown() -> ! {
    scall(SBI_SHUTDOWN, 0, 0, 0);
    panic!("It should shutdown!");
}

pub fn put_char(c: usize) {
    scall(SBI_CONSOLE_PUTCHAR, c, 0, 0);
}

pub fn set_timer(timer: usize) {
    scall(SBI_SET_TIMER, timer, 0, 0);
}
const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_GET_TIME: usize = 169;

mod fs;
mod process;

use fs::*;
use process::*;
// 处理来自 apps 的 trap
pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    match syscall_id {
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_YIELD => sys_yield(),
        SYSCALL_GET_TIME => sys_get_time(),
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}

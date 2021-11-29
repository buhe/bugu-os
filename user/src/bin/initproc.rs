#![no_std]
#![no_main]

#[macro_use]
extern crate user;

use user::{
    fork,
    wait,
    exec,
    yield_,
};

#[no_mangle]
fn main() -> i32 {
   if fork() == 0 {
        exec("shell\0", &[0 as *const u8]);
    } else {
        loop {
            let mut exit_code: i32 = 0;
            let pid = wait(&mut exit_code);
            if pid == -1 {
                yield_();
                continue;
            }
            println!(
                "[initproc] Released a zombie process, pid={}, exit_code={}",
                pid,
                exit_code,
            );
        }
    }
    0
}
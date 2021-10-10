#![no_std]
#![no_main]
#![feature(llvm_asm)]

#[macro_use]
extern crate user;

#[no_mangle]
fn main() -> i32 {
    println!("Hello OS from app@v-mem");
    0
}
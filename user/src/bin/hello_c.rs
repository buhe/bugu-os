#![no_std]
#![no_main]


extern crate user;
#[link(name = "h", kind = "static")]
extern "C" {
    fn print2();
}

#[no_mangle]
fn main() -> i32 {
    unsafe{ print2();}
    0
}
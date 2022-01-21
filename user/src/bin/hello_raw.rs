#![no_std]
#![no_main]


extern crate user;
#[link(name = "example3", kind = "static")]
extern "C" {
    fn main();
}
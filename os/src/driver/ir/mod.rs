use k210_soc::{fpioa::{self, io}, gpio, gpiohs};
use lazy_static::*;
use spin::Mutex;

use crate::driver::ir::rx::IRrev;

mod rx;
mod tx;

// pub fn io_init(p: Peripherals, clock: &Clocks) {}

lazy_static! {
    pub static ref IRRX: Mutex<IRrev> = {
        let rev = IRrev::new();
        fpioa::set_function(io::IO32, fpioa::function::GPIOHS1);
        gpiohs::set_direction(1, gpio::direction::INPUT);
        // println!("ir rev is {}", rx.try_read().unwrap());
        Mutex::new(rev)
    };
}
pub fn init() {
    // init 31 32 pins
    // rx::init(31);
    IRRX.lock();
}

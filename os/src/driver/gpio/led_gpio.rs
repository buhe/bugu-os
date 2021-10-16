use k210_soc::{
    fpioa::{self, io},
    gpio,
};

use crate::driver::gpio::driver;

pub fn init() {
    // led b 映射到 gpio 0
    fpioa::set_function(io::LED_G, fpioa::function::GPIO0);
    // gpiohs 设置 0 为输出
    driver::set_direction(0, gpio::direction::OUTPUT);
    // gpiohs 0 为 false , false 为点亮
    driver::set_pin(0, false);
}

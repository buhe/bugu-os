// 外部的 led

use k210_soc::{
    fpioa::{self, io},
    gpio, gpiohs,
};

pub fn init() {
    // led b 映射到 gpiohs 0
    // 9
    fpioa::set_function(io::BPSK_P, fpioa::function::GPIOHS0);
    // 10
    fpioa::set_function(io::BPSK_N, fpioa::function::GPIOHS1);
    // gpiohs 设置 0 为输出
    gpiohs::set_direction(0, gpio::direction::OUTPUT);
    gpiohs::set_direction(1, gpio::direction::OUTPUT);

    gpiohs::set_pin(0, false);
    gpiohs::set_pin(1, true);
}

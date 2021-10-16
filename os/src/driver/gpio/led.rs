use k210_soc::{
    fpioa::{self, io},
    gpio, gpiohs,
};

pub fn init() {
    // led b 映射到 gpiohs 0
    // io::LED_B 为物理 pin
    fpioa::set_function(io::LED_B, fpioa::function::GPIOHS0);
    // gpiohs 设置 0 为输出
    gpiohs::set_direction(0, gpio::direction::OUTPUT);
    // gpiohs 0 为 false , false 为点亮
    gpiohs::set_pin(0, false);
}

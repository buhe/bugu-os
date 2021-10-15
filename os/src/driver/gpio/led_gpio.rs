use k210_soc::{
    fpioa::{self, io},
    gpio, gpiohs,
};

pub fn init() {
    // led b 映射到 gpiohs 0
    fpioa::set_function(io::LED_R, fpioa::function::GPIOHS5);
    // gpiohs 设置 0 为输出
    gpiohs::set_direction(5, gpio::direction::OUTPUT);
    // gpiohs 0 为 false , false 为点亮
    gpiohs::set_pin(5, false);
    println!("0 is {}", gpiohs::get_pin(5));
}

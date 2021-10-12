use k210_soc::{
    fpioa::{self, io},
    gpio, gpiohs,
};

pub fn init() {
    fpioa::set_function(io::LED_B, fpioa::function::GPIOHS0);
    gpiohs::set_direction(0, gpio::direction::OUTPUT);
    gpiohs::set_pin(0, false);
    println!("0 is {}", gpiohs::get_pin(0));
}

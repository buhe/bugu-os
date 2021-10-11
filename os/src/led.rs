use k210_soc::{
    fpioa::{self, io},
    gpio, gpiohs,
};

pub fn init() {
    fpioa::set_function(io::LED_B, fpioa::function::GPIO3);
    gpiohs::set_direction(3, gpio::direction::OUTPUT);
    gpiohs::set_pin(3, true);

    // gpio_init();
    // gpio_set_drive_mode(3, GPIO_DM_OUTPUT);
    // gpio_pin_value_t value = GPIO_PV_HIGH;
    // gpio_set_pin(3, value);
}

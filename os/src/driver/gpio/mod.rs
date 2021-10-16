mod led_gpio;
mod driver;
pub fn init() {
    led_gpio::init();
}

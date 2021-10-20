mod driver;
mod led_gpio;
pub fn init() {
    led_gpio::init();
    println!("inited led");
}

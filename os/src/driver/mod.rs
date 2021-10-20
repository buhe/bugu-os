mod gpio;
mod lcd;
pub fn init() {
    gpio::init();
    lcd::init();
}

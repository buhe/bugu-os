mod gpio;
mod lcd;
pub fn init() {
    gpio::init();
    lcd::init();
}
pub use lcd::print_with_lcd;

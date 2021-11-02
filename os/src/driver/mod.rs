mod gpio;
mod lcd;
mod ir;
pub fn init() {
    gpio::init();
    lcd::init();
    ir::init();
}
pub use lcd::print_with_lcd;

mod gpio;
// mod ir;
mod lcd;
mod network;
pub fn init() {
    gpio::init();
    lcd::init();
    // ir::init();
    // network::init();
}
pub use lcd::print_with_lcd;

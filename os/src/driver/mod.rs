mod gpio;
mod block;
// mod ir;
mod lcd;
// mod network;
pub fn init() {
    gpio::init();
    lcd::init();
    // ir::init();
    // network::init();
}
pub use lcd::print_with_lcd;
pub use lcd::flush;
pub use block::BLOCK_DEVICE;

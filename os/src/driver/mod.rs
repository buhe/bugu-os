mod gpio;
mod block;
// mod ir;
mod epaper;
// mod network;
pub fn init() {
    gpio::init();
    epaper::init();
    // lcd::init();
    // ir::init();
    // network::init();
}
pub use epaper::print_with_lcd;
pub use epaper::flush;
pub use block::BLOCK_DEVICE;

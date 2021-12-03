mod gpio;
mod block;
// mod ir;
mod lcd;
mod network;
pub fn init() {
    // usleep(2000000);
    gpio::init();
    // usleep(2000000);
    lcd::init();
    // ir::init();
    // usleep(2000000);
    network::init();
}
use k210_soc::sleep::usleep;
pub use lcd::print_with_lcd;
pub use lcd::flush;
pub use block::BLOCK_DEVICE;

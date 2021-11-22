mod gpio;
mod block;
// mod ir;
// mod lcd;
// mod network;
pub fn init() {
    gpio::init();
    // lcd::init();
    // ir::init();
    // network::init();
    block::init();
}
// pub use lcd::print_with_lcd;
// pub use lcd::flush;

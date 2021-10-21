mod gpio;
mod lcd;
pub fn init() {
    gpio::init();
}

pub use lcd::print_lcd;

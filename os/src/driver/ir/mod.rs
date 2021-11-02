mod rx;
mod tx;

pub fn init() {
    // init 31 32 pins
    rx::init(31);
}
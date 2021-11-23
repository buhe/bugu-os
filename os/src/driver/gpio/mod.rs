mod driver;
mod led;
pub fn init() {
    led::init();
    println!("inited led");
}

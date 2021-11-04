mod driver;
mod led_ex;
pub fn init() {
    led_ex::init();
    println!("inited led");
}

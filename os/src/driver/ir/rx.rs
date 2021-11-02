use core::convert::TryInto;

use k210_soc::{fpioa::{self}, gpio, gpiohs, sleep::usleep, sysctl};

pub fn init(pin: usize) {
    let rev = IRrev::new(pin);
    rev.io_init();
}

struct IRrev{
    pin: usize
}

impl IRrev {
    fn new(pin: usize) -> Self {
        Self {pin}
    }

    fn io_init(&self) {
        sysctl::clock_enable(sysctl::clock::UART2);
        sysctl::reset(sysctl::reset::UART2);
        fpioa::set_function(self.pin, fpioa::function::UART2_RX);

        gpiohs::set_direction(self.pin.try_into().unwrap(), gpio::direction::INPUT);
        println!("sleep...");
        usleep(10*1000*1000);
        println!("sleep end.");
        println!("ir rev is {}", gpiohs::get_pin(self.pin.try_into().unwrap()));
    }
}
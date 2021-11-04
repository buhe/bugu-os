use k210_hal::{clock::Clocks, prelude::*};
use k210_pac::Peripherals;
use k210_soc::{
    fpioa::{self, io},
    // sleep::usleep,
    sysctl,
};
use lazy_static::*;
use spin::Mutex;

use crate::driver::ir::rx::IRrev;

mod rx;
mod tx;
const DEFAULT_BAUD: u32 = 115_200;

// pub fn io_init(p: Peripherals, clock: &Clocks) {}

lazy_static! {
    pub static ref IRRX: Mutex<IRrev> = {
        let p = Peripherals::take().unwrap();
        sysctl::pll_set_freq(sysctl::pll::PLL0, 800_000_000).unwrap();
        sysctl::pll_set_freq(sysctl::pll::PLL1, 300_000_000).unwrap();
        sysctl::pll_set_freq(sysctl::pll::PLL2, 45_158_400).unwrap();
        let clocks = Clocks::new();
        let rev = IRrev::new();
        // io_init(&p, &clocks);
        sysctl::clock_enable(sysctl::clock::UART2);
        sysctl::reset(sysctl::reset::UART2);
        fpioa::set_function(io::I2C1_SDA, fpioa::function::UART2_RX);
        let ir = p.UART2.configure(DEFAULT_BAUD.bps(), &clocks);
        let (_, mut rx) = ir.split();
    //     // gpiohs::set_direction(self.pin.try_into().unwrap(), gpio::direction::INPUT);
    //     // println!("sleep...");
    //     // usleep(10*1000*1000);
    //     // println!("sleep end.");
        // println!("ir rev is {}", rx.try_read().unwrap());
        Mutex::new(rev)
    };
}
pub fn init() {
    // init 31 32 pins
    // rx::init(31);
    IRRX.lock();
}

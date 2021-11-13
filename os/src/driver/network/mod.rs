use k210_hal::{prelude::*, serial::{Serial, Tx}};
use k210_pac::{Peripherals, UART1};
use k210_soc::{fpioa::{self, io}, gpio, gpiohs, sleep::usleep, sysctl};
use self::{handler::SerialNetworkHandler, traits::Write};


pub mod handler;
pub mod response;
pub mod traits;
mod util;

const DEFAULT_BAUD: u32 = 115_200;
struct WA {
    s: Tx<UART1>,
}

impl WA {
    fn new(s: Tx<UART1>) -> Self {
        Self {
            s,
        }
    }
}
impl Write for WA{
    type Error = ();
    fn write_all(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        for &b in buf {
             self.s.try_write(b).unwrap();
        }
        Ok(())
    }
}

fn init_io(){
    sysctl::clock_enable(sysctl::clock::UART1);
    sysctl::reset(sysctl::reset::UART1);
    fpioa::set_function(io::WIFI_RX, fpioa::function::UART1_TX);
    fpioa::set_function(io::WIFI_TX, fpioa::function::UART1_RX);
    fpioa::set_function(io::WIFI_EN, fpioa::function::GPIOHS8);
    fpioa::set_io_pull(io::WIFI_EN, fpioa::pull::DOWN);
    gpiohs::set_direction(8, gpio::direction::OUTPUT);
    gpiohs::set_pin(8, true);
}

pub fn init(){
     let p = Peripherals::take().unwrap();
    sysctl::pll_set_freq(sysctl::pll::PLL0, 800_000_000).unwrap();
    sysctl::pll_set_freq(sysctl::pll::PLL1, 300_000_000).unwrap();
    sysctl::pll_set_freq(sysctl::pll::PLL2, 45_158_400).unwrap();
    let clocks = k210_hal::clock::Clocks::new();
      usleep(200000);
     init_io();
     
     let uart1 = p.UART1.configure(DEFAULT_BAUD.bps(), &clocks);
     let (tx, mut rx) = uart1.split();
     let mut wa = WA::new(tx);
     let mut h = SerialNetworkHandler::new(&mut wa, "" .as_bytes(), "".as_bytes());
     usleep(1_000_000);
     h.start(false).unwrap();
     usleep(2 * 1_000_000);
     h.list().unwrap();
     println!("inited netword");
     loop{
         let u = rx.try_read().unwrap();
         println!("{}", u as char);
     }
}
use k210_hal::{prelude::*, serial::{Serial, Tx}};
use k210_pac::{Peripherals, UART1};
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
pub fn init(){
     let p = Peripherals::take().unwrap();
     let clocks = k210_hal::clock::Clocks::new();
     let uart1 = p.UART1.configure(DEFAULT_BAUD.bps(), &clocks);
     let (tx, rx) = uart1.split();
     let mut wa = WA::new(tx);
     let h = SerialNetworkHandler::new(&mut wa, "" .as_bytes(), "".as_bytes());
}
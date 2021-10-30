use core::fmt;

use k210_hal::pac::Peripherals;
use k210_hal::prelude::*;
use k210_pac::SPI0;
use k210_soc::{
    dmac::DMACExt,
    fpioa::{self, io},
    sleep::usleep,
    spi::{SPIExt, SPIImpl},
    sysctl::{self, dma_channel},
};
use lazy_static::*;
use spin::Mutex;

use self::{
    console::{Console, ScreenImage, DISP_PIXELS},
    st7789v::{LCD, LCDHL},
};

mod st7789v;

mod color;
mod coord;
mod cp437;
mod cp437_8x8;
mod lcd_colors;
mod palette_xterm256;
// 用 LCD 输出
mod console;
static mut IMAGE: ScreenImage = [0; DISP_PIXELS / 2];
pub fn init() {
    DRIVER.lock();
}

/** Connect pins to internal functions */
fn io_mux_init() {
    /* Init SPI IO map and function settings */
    fpioa::set_function(io::LCD_RST, fpioa::function::gpiohs(st7789v::RST_GPIONUM));
    fpioa::set_io_pull(io::LCD_RST, fpioa::pull::DOWN); // outputs must be pull-down
    fpioa::set_function(io::LCD_DC, fpioa::function::gpiohs(st7789v::DCX_GPIONUM));
    fpioa::set_io_pull(io::LCD_DC, fpioa::pull::DOWN);
    fpioa::set_function(io::LCD_CS, fpioa::function::SPI0_SS3);
    fpioa::set_function(io::LCD_WR, fpioa::function::SPI0_SCLK);

    sysctl::set_spi0_dvp_data(true);
}

/** Set correct voltage for pins */
fn io_set_power() {
    /* Set dvp and spi pin to 1.8V */
    sysctl::set_power_mode(sysctl::power_bank::BANK6, sysctl::io_power_mode::V18);
    sysctl::set_power_mode(sysctl::power_bank::BANK7, sysctl::io_power_mode::V18);
}

lazy_static! {
    pub static ref CONSOLE: Mutex<Console> = {
        let console: Console =
                Console::new(&cp437_8x8::FONT, None);

            /* Make a border */
            // let fg = Color::new(0x80, 0x40, 0x40);
            // let bg = Color::new(0x00, 0x00, 0x00);
            // // Sides
            // for x in 1..console.width() - 1 {
            //     console.put(x, 0, fg, bg, '─');
            //     console.put(x, console.height() - 1, fg, bg, '─');
            // }
            // for y in 1..console.height() - 1 {
            //     console.put(0, y, fg, bg, '│');
            //     console.put(console.width() - 1, y, fg, bg, '│');
            // }

            Mutex::new(console)
    };
    pub static ref DRIVER: Mutex<LCD<SPIImpl<SPI0>>> = {
        let p = Peripherals::take().unwrap();
    sysctl::pll_set_freq(sysctl::pll::PLL0, 800_000_000).unwrap();
    sysctl::pll_set_freq(sysctl::pll::PLL1, 300_000_000).unwrap();
    sysctl::pll_set_freq(sysctl::pll::PLL2, 45_158_400).unwrap();
    // Configure clocks (TODO)
    // let clocks = k210_hal::clock::Clocks::new();
    // sleep a bit to let clients connect
    usleep(200000);

    io_mux_init();
    io_set_power();

     /* LCD init */
    let dmac = p.DMAC.configure();
    let spi = p.SPI0.constrain();
    let mut lcd = LCD::new(spi, dmac, dma_channel::CHANNEL0);
    lcd.init();
    lcd.set_direction(st7789v::direction::YX_LRUD);
    // lcd.clear(lcd_colors::BLUE);
        Mutex::new(lcd)
    };
}

pub fn print_with_lcd(args: fmt::Arguments) {
    CONSOLE.lock().write_fmt(args).unwrap();
    unsafe {
        CONSOLE.lock().render(&mut IMAGE);
        DRIVER.lock().flush(&IMAGE);
    }
}

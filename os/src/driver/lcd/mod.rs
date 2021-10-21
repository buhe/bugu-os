

use core::fmt::{self, Write};

use lazy_static::*;

use crate::driver::lcd::console::Console;



mod st7789v;

mod coord;
mod palette_xterm256;
mod lcd_colors;
mod color;
mod cp437;
mod cp437_8x8;
// 用 LCD 输出
pub mod console;

use spin::Mutex;
use alloc::sync::Arc;

lazy_static! {
    pub static ref LCD_DRIVER : Arc<Mutex<Console>> = Arc::new(Mutex::new(Console::init()));
}

pub fn print_lcd(args: fmt::Arguments) {
    LCD_DRIVER.lock().write_fmt(args).unwrap();
}




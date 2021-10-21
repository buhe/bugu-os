use core::fmt::{self, Write};

use crate::{driver::print_lcd, scall_sbi::put_char};

struct STDOUT;

impl Write for STDOUT {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            put_char(c as usize);
        }
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    STDOUT.write_fmt(args).unwrap();
    // print_lcd(args);
}

#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}

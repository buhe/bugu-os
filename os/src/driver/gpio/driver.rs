#![allow(unused)]

use k210_soc::gpio;
use k210_soc::utils::{get_bit, set_bit};

/** Set input/output direction for a GPIOHS pin */
pub fn set_direction(pin: u8, direction: gpio::direction) {
    unsafe {
        let ptr = k210_pac::GPIO::ptr();
        (*ptr)
            .direction
            .modify(|r, w| w.bits(set_bit(r.bits(), pin, direction == gpio::direction::OUTPUT)));
    }
}

/** Set output value for a GPIOHS pin */
pub fn set_pin(pin: u8, value: bool) {
    unsafe {
        let ptr = k210_pac::GPIO::ptr();
        (*ptr)
            .data_output
            .modify(|r, w| w.bits(set_bit(r.bits(), pin, value)));
    }
}

// pub fn get_pin(pin: u8) -> bool {
//     unsafe {
//         let ptr = k210_pac::GPIOHS::ptr();
//         get_bit((*ptr).input_val.read().bits(), pin)
//     }
// }

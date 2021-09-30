use core::panic::PanicInfo;

use crate::scall_sbi::shutdown;

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    shutdown();
}

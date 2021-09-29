#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
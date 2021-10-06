#![feature(asm)]
#![feature(global_asm)]
#![feature(alloc_error_handler)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![no_std]
#![no_main]
#![reexport_test_harness_main = "test_main"]

// use device_tree::DeviceTree;

#[macro_use]
extern crate bitflags;

extern crate alloc;
#[macro_use]
mod console;
mod config;
mod heap;
mod lang;
mod mmu;
mod scall_sbi;
mod task;
mod trap;

global_asm!(include_str!("stack.asm"));
global_asm!(include_str!("link_app.S"));

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
}

#[no_mangle]
extern "C" fn rust_main(hartid: usize, device_tree_paddr: usize) -> ! {
    println!("hart id is {}", hartid);
    println!("dtb addr is 0x{:x}", device_tree_paddr);
    #[repr(C)]
    struct DtbHeader {
        be_magic: u32,
        be_size: u32,
    }
    let header = unsafe { &*(device_tree_paddr as *const DtbHeader) };
    // from_be 是大小端序的转换（from big endian）
    let magic = u32::from_be(header.be_magic);
    println!("check magic is 0xd00dfeed, magic is 0x{:x}", magic);
    const DEVICE_TREE_MAGIC: u32 = 0xd00dfeed;
    assert_eq!(magic, DEVICE_TREE_MAGIC);
    let size = u32::from_be(header.be_size);
    let _dtb_data = unsafe { core::slice::from_raw_parts(device_tree_paddr as *const u8, size as usize) };
    // let dt = DeviceTree::load(dtb_data).expect("failed to parse device tree");
    // DeviceTree::load is not adpator k210
    println!("dt size is {:#?}", size);
    
    clear_bss();
    heap::init();
    mmu::init();
    trap::init();
    task::init();

    #[cfg(test)]
    test_main();

    task::run();
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
        println!("[ok]");
    }
}

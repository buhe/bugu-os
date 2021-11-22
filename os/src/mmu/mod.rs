mod address;
mod frame_allocator;
mod memory_set;
mod page_table;
mod user_buffer;

use page_table::PTEFlags;
use address::VPNRange;
pub use address::{PhysAddr, VirtAddr, PhysPageNum, VirtPageNum, StepByOne};
pub use frame_allocator::{FrameTracker, frame_alloc, frame_dealloc,};
pub use page_table::{
    PageTable,
    PageTableEntry,
    translated_byte_buffer,
    translated_str,
    translated_ref,
    translated_refmut,
};
pub use memory_set::{MemorySet, KERNEL_SPACE, MapPermission, kernel_token};
pub use memory_set::remap_test;
pub use user_buffer::{UserBuffer,UserBufferIterator};

pub fn init() {
    // 启动分页器
    frame_allocator::init_frame_allocator();
    // 启动虚拟内存
    KERNEL_SPACE.lock().activate();
}

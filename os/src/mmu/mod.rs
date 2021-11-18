mod address;
mod frame_allocator;
mod memory_set;
mod page_table;
mod user_buffer;

pub use address::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum};
use address::{StepByOne, VPNRange};
pub use frame_allocator::{frame_alloc, FrameTracker};
pub use memory_set::{MapPermission, MemorySet, KERNEL_SPACE};
pub use page_table::{translated_byte_buffer, translated_refmut, translated_str, PageTableEntry};
use page_table::{PTEFlags, PageTable};
pub use user_buffer::UserBuffer;

pub fn init() {
    // 启动分页器
    frame_allocator::init_frame_allocator();
    // 启动虚拟内存
    KERNEL_SPACE.lock().activate();
}

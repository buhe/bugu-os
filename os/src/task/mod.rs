use crate::{config::{TRAP_CONTEXT}, mmu::{KERNEL_SPACE, MemorySet, PhysPageNum, VirtAddr}, trap::{TrapContext, trap_handler, trap_return}};
use core::{cell::RefCell, usize};
use lazy_static::*;

struct AppManager {
    inner: RefCell<AppManagerInner>,
}
struct AppManagerInner {
    app_start: [usize; 2],
    pub token: usize,
    trap_cx_ppn: PhysPageNum,
}
unsafe impl Sync for AppManager {}

impl AppManagerInner {
    pub fn print_app_info(&self) {
        println!(
            "[kernel] app_{} [{:#x}, {:#x})",
            0, self.app_start[0], self.app_start[1]
        );
    }

    unsafe fn load_app(&mut self) {
        // clear app area
        let app_src = core::slice::from_raw_parts(
            self.app_start[0] as *const u8,
            self.app_start[1] - self.app_start[0],
        );
        // 入口点是从 elf 加载的
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(&app_src);
        // 通过查表, 从虚拟页获得实际的物理页
        self.trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();
        // 因为目前还是单任务, 暂没有 task 和 TaskContext, 只有 trap
        // 获取 trap context 的指针并赋值
        let trap_cx = self.trap_cx_ppn.get_mut();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.lock().token(),
            // kernel_stack_top,
            trap_handler as usize,
        );
    }
}

lazy_static! {
    static ref APP_MANAGER: AppManager = AppManager {
        inner: RefCell::new({
            extern "C" {
                fn _num_app();
            }
            let num_app_ptr = _num_app as usize as *const usize;
            let num_app = unsafe { num_app_ptr.read_volatile() };
            let mut app_start: [usize; 2] = [0; 2];
            let app_start_raw: &[usize] =
                unsafe { core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1) };
            app_start[..=num_app].copy_from_slice(app_start_raw);
            AppManagerInner { app_start,token: 0, trap_cx_ppn: PhysPageNum(0) }
        }),
    };
}

pub fn init() {
    print_app_info();
}

pub fn print_app_info() {
    APP_MANAGER.inner.borrow().print_app_info();
}

pub fn run() -> ! {
    // 加载 app 到虚拟地址
    unsafe {
        APP_MANAGER.inner.borrow_mut().load_app();
    }
    // 调用 restore 启动 app
    trap_return();
}


pub fn current_user_token() -> usize {
    APP_MANAGER.inner.borrow().token
}
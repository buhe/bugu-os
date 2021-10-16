## 唤醒应用

第一个应用都是内核运行的，下面就来看看怎么加载并运行应用

```rust
    unsafe fn load_app(&mut self) {
        // clear app area
        let app_src = core::slice::from_raw_parts(
            self.app_start[0] as *const u8,
            self.app_start[1] - self.app_start[0],
        );
        // 入口点是从 elf 加载的
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(&app_src);
        self.token = memory_set.token();
        // 通过查表, 从虚拟页获得实际的物理页
        self.trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();
        // 因为目前还是单任务, 暂没有 task 和 TaskContext, 只有 trap
        // 内核栈单纯用于内核的函数调用
        // map a kernel-stack in kernel space
        let (kernel_stack_bottom, kernel_stack_top) = kernel_stack_position(0);
        KERNEL_SPACE.lock().insert_framed_area(
            kernel_stack_bottom.into(),
            kernel_stack_top.into(),
            MapPermission::R | MapPermission::W,
        );
        // 获取 trap context 的指针并赋值
        let trap_cx = self.trap_cx_ppn.get_mut();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.lock().token(),
            kernel_stack_top,
            trap_handler as usize,
        );
    }
}

pub fn run() -> ! {
    // 加载 app 到虚拟地址
    unsafe {
        APP_MANAGER.inner.borrow_mut().load_app();
    }
    // 调用 restore 启动 app
    trap_return();
}
```

1. load_app 从内核的数据段加载应用的 elf 
2. 因为 TrapContext 在每个应用固定的虚拟地址，所以通过手动查映射可以查到
3. 初始化 TrapContext ，通过 elf 中的 entry_point 设置 sepc 也就是 trap 后执行的第一行代码
4. 跳到 restore 的地址，执行汇编

## 打印

铺垫了那么多，要在用户态打印就必须 trap 到内核态，要 trap 就必须满足

- 首先要保存寄存器到 trap context ，不同的是它在用户地址空间，因为只有一个 sscratch 做中转、用来保存 sp 了，没有中转 token(保存地址空间页表的地址)的寄存器
- 还要把地址空间切换到内核，因为需要的数据结构都在内核态，所以内核栈也在内核

先来满足 2 ，我们建立内核和应用地址空间

### 建立内核地址空间

```rust
pub fn new_kernel() -> Self {
        let mut memory_set = Self::new_bare();
        // map trampoline
        memory_set.map_trampoline();
        // map kernel sections
        println!(".text [{:#x}, {:#x})", stext as usize, etext as usize);
        println!(".rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
        println!(".data [{:#x}, {:#x})", sdata as usize, edata as usize);
        println!(
            ".bss [{:#x}, {:#x})",
            sbss_with_stack as usize, ebss as usize
        );
        println!("mapping .text section");
        memory_set.push(
            MapArea::new(
                (stext as usize).into(),
                (etext as usize).into(),
                MapType::Identical,
                MapPermission::R | MapPermission::X,
            ),
            None,
        );
        println!("mapping .rodata section");
        memory_set.push(
            MapArea::new(
                (srodata as usize).into(),
                (erodata as usize).into(),
                MapType::Identical,
                MapPermission::R,
            ),
            None,
        );
        println!("mapping .data section");
        memory_set.push(
            MapArea::new(
                (sdata as usize).into(),
                (edata as usize).into(),
                MapType::Identical,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        println!("mapping .bss section");
        memory_set.push(
            MapArea::new(
                (sbss_with_stack as usize).into(),
                (ebss as usize).into(),
                MapType::Identical,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        println!("mapping physical memory");
        memory_set.push(
            MapArea::new(
                (ekernel as usize).into(),
                MEMORY_END.into(),
                MapType::Identical,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );

        for pair in MMIO {
            memory_set.push(
                MapArea::new(
                    (*pair).0.into(),
                    ((*pair).0 + (*pair).1).into(),
                    MapType::Identical,
                    MapPermission::R | MapPermission::W,
                ),
                None,
            );
        }
        memory_set
    }
```

把内核各个数据段用恒等映射对应起来，只是没有映射内核栈，因为内核栈一个应用一个，在映射应用的时候映射。最后一部分是映射 MMIO 的地址，会在编写驱动的时候用上。

### 建立应用地址空间

```rust
pub fn from_elf(elf_data: &[u8]) -> (Self, usize, usize) {
        let mut memory_set = Self::new_bare();
        // map trampoline
        memory_set.map_trampoline();
        // map program headers of elf, with U flag
        let elf = xmas_elf::ElfFile::new(elf_data).unwrap();
        let elf_header = elf.header;
        let magic = elf_header.pt1.magic;
        assert_eq!(magic, [0x7f, 0x45, 0x4c, 0x46], "invalid elf!");
        let ph_count = elf_header.pt2.ph_count();
        let mut max_end_vpn = VirtPageNum(0);
        for i in 0..ph_count {
            let ph = elf.program_header(i).unwrap();
            if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
                let start_va: VirtAddr = (ph.virtual_addr() as usize).into();
                let end_va: VirtAddr = ((ph.virtual_addr() + ph.mem_size()) as usize).into();
                let mut map_perm = MapPermission::U;
                let ph_flags = ph.flags();
                if ph_flags.is_read() {
                    map_perm |= MapPermission::R;
                }
                if ph_flags.is_write() {
                    map_perm |= MapPermission::W;
                }
                if ph_flags.is_execute() {
                    map_perm |= MapPermission::X;
                }
                let map_area = MapArea::new(start_va, end_va, MapType::Framed, map_perm);
                max_end_vpn = map_area.vpn_range.get_end();
                memory_set.push(
                    map_area,
                    Some(&elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize]),
                );
            }
        }
        // map user stack with U flags
        let max_end_va: VirtAddr = max_end_vpn.into();
        let mut user_stack_bottom: usize = max_end_va.into();
        // guard page
        user_stack_bottom += PAGE_SIZE;
        let user_stack_top = user_stack_bottom + USER_STACK_SIZE;
        memory_set.push(
            MapArea::new(
                user_stack_bottom.into(),
                user_stack_top.into(),
                MapType::Framed,
                MapPermission::R | MapPermission::W | MapPermission::U,
            ),
            None,
        );
        // map TrapContext
        memory_set.push(
            MapArea::new(
                TRAP_CONTEXT.into(),
                TRAMPOLINE.into(),
                MapType::Framed,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        (
            memory_set,
            user_stack_top,
            elf.header.pt2.entry_point() as usize,
        )
    }
```

1. 解析 elf ，根据 elf 映射应用的各个数据段，
2. 映射用户栈
3. 映射 TrapContext 

### Trap

再满足 1 ，TrapContext 在应用地址空间的跳板后面，在汇编里保存寄存器到 TrapContext 前面介绍过。

![app-as-full](https://tva1.sinaimg.cn/large/008i3skNgy1gvh34duxtvj60wb0ibwft02.jpg)

### 跳板

跳板是干嘛的？其实 trap_handler 就保存在其中，它的权限也没有 U ，应用是不能访问的。

```rust
    fn map_trampoline(&mut self) {
        self.page_table.map(
            VirtAddr::from(TRAMPOLINE).into(),
            PhysAddr::from(strampoline as usize).into(),
            PTEFlags::R | PTEFlags::X,
        );
    }
```

跳板在内核和应用中都有，且虚拟地址相同，保证在 trap 的时候切换地址空间前后都能访问到 trap_handler ，也就是相同虚拟地址映射到相同的物理地址。注意，map_trampoline 没有使用  MapArea::new ，隐含是恒等映射，不需要分配物理内存，只要在 trap.asm 中指定     .section .text.trampoline 就可以，会把 trap.asm 的数据放在跳板中。

### 改造系统调用--字符输出



具体代码请参考 https://github.com/buhe/bugu/tree/6

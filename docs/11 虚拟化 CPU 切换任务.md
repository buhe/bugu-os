### __switch

```asm
.altmacro
.macro SAVE_SN n
    sd s\n, (\n+1)*8(sp)
.endm
.macro LOAD_SN n
    ld s\n, (\n+1)*8(sp)
.endm
    .section .text
    .globl __switch
__switch:
    # __switch(
    #     current_task_cx_ptr2: &*const TaskContext,
    #     next_task_cx_ptr2: &*const TaskContext
    # )
    # push TaskContext to current sp and save its address to where a0 points to
    addi sp, sp, -13*8
    # sd 是寄存器到内存
    sd sp, 0(a0)
    # fill TaskContext with ra & s0-s11
    sd ra, 0(sp)
    .set n, 0
    .rept 12
        SAVE_SN %n
        .set n, n + 1
    .endr
    # ready for loading TaskContext a1 points to
    # a1 -> sp
    ld sp, 0(a1)
    # load registers in the TaskContext
    ld ra, 0(sp)
    .set n, 0
    .rept 12
        LOAD_SN %n
        .set n, n + 1
    .endr
    # pop TaskContext
    addi sp, sp, 13*8
    ret
```

__switch 函数主要保存被调用者的寄存器到内存中的 TaskContext 中，而后改变 sp 和 ra 寄存器。

- sp 寄存器指定了新任务的堆栈位置
- ra 寄存器其实是 return address ，函数返回的地址，指定了新任务下一步要执行的指令

切换了它们两个寄存器，就切换了任务。

### trap_return

```rust
#[no_mangle]
pub fn trap_return() -> ! {
    set_user_trap_entry();
    let trap_cx_ptr = TRAP_CONTEXT;
    let user_satp = current_user_token();
    extern "C" {
        fn __alltraps();
        fn __restore();
    }
    let restore_va = __restore as usize - __alltraps as usize + TRAMPOLINE;
    unsafe {
        asm!(
            "fence.i",
            "jr {restore_va}",
            restore_va = in(reg) restore_va,
            in("a0") trap_cx_ptr,
            in("a1") user_satp,
            options(noreturn)
        );
    }
    // panic!("Unreachable in back_to_user!");
}
```

为什么不直接调用 __restore 呢？

因为 TrapContext 在应用的虚拟内存里，要拿到应用的 token 和 TrapContext 的虚拟地址，__restore 先通过 token 切换地址空间再访问 TrapContext 。

### 抢占式调度

为什么使用抢占式调度？协作就像靠自觉，抢占就像排队，排队整体效率提高了，虽然牺牲了个人利益。

抢占式调度采用时钟中断实现，时钟中断就是常说的时间片，时钟中断其实是 SBI 提供的。到了固定的时间触发时钟中断，进而进入中断处理程序，中断处理程序发现是时钟中断，切换到下一个任务来完成抢占式调度。

```rust
pub fn get_time() -> usize {
    time::read()
}

pub fn get_time_ms() -> usize {
    time::read() / (CLOCK_FREQ / MSEC_PER_SEC)
}

pub fn set_next_trigger() {
    set_timer(get_time() + CLOCK_FREQ / TICKS_PER_SEC);
}
```


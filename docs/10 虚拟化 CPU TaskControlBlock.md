首先，为什么要虚拟化 CPU 呢？

CPU 是用来计算的，当应用程序越来越复杂，交互性也越来越强，应用要更多的跟外设交互，产生 IO 。产生 IO 的时候 CPU 处于等待状态，浪费掉了，如果这时候切换到另一个需要计算的应用，这样切换的成本低于浪费掉的就是划算的。

而不能因为 CPU 的切换就需要应用改变，这种切换最好对应用来讲是透明的。这就分为协作式和抢占式，协作式需要主动调用 yield 来放弃 CPU ，抢占式通过时钟来被动切换进而达到对应用透明的目的。抢占式通过 CPU 的快速切换来让应用以为自己独占 CPU 。

应用程序在运行时被抽象成任务，那些 CPU 运行需要的信息保存在任务信息中。

### 任务信息

任务信息是 TaskControlBlock

```rust
pub struct TaskControlBlock {
    pub task_cx_ptr: usize,
    pub task_status: TaskStatus,
    pub memory_set: MemorySet,
    pub trap_cx_ppn: PhysPageNum,
    pub base_size: usize,
}
```

- task_cx_ptr 是任务上下文的指针
- task_status 是任务状态
- memory_set 是读取 elf 获取的地址空间
- trap_cx_ppn 是 trap 上下文的物理页

rust 通过操纵 TaskControlBlock 来间接操纵任务，我们从总体视角看看

- 定时器每隔一段时间产生一个中断，进入 trap 
- trap.asm 在虚拟内存的跳板上，每个应用和内核都是相同的映射 stvec::write(TRAMPOLINE as usize, TrapMode::Direct); 通过这行代码，trap 的处理设定为从跳板的开头开始
- 跳板的开头是 __alltraps ，保存寄存器后跳到 rust 代码的 trap_handler 

``` rust
  Trap::Interrupt(Interrupt::SupervisorTimer) => {
            set_next_trigger();
            suspend_current_and_run_next();
        }
```

- 分支切换到下个任务，suspend_current_and_run_next 调用 run_next_task 。

```rust
    fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.borrow_mut();
            let current = inner.current_task;
            inner.tasks[next].task_status = TaskStatus::Running;
            inner.current_task = next;
            let current_task_cx_ptr2 = inner.tasks[current].get_task_cx_ptr2();
            let next_task_cx_ptr2 = inner.tasks[next].get_task_cx_ptr2();
            core::mem::drop(inner);
            unsafe {
                __switch(current_task_cx_ptr2, next_task_cx_ptr2);
            }
        } else {
            panic!("All applications completed!");
        }
    }
```

- run_next_task 具体调用 __switch 切换任务
- __switch 其实就是一个函数调用
  - 保存被调用寄存器
  - 切换 sp 和 ra 来达到切换应用到目的
- 最后调用 trap_return 间接调用 __restore

再一个个的看，处理 trap 和上节相同，重点是如何切换任务，也就是 __switch 


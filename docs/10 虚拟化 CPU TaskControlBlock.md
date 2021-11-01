首先，为什么要虚拟化 CPU 呢？

CPU 是用来计算的，当应用程序越来越复杂，交互性也越来越强，应用要更多的跟外设交互，产生 IO 。产生 IO 的时候 CPU 处于等待状态，浪费掉了，如果这时候切换到另一个需要计算的应用，这样切换的成本低于浪费掉的就是划算的。

而不能因为 CPU 的切换就需要应用改变，这种切换最好对应用来讲是透明的。这就分为协作式和抢占式，协作式需要主动调用 yield 来放弃 CPU ，抢占式通过时钟来被动切换进而达到对应用透明的目的。抢占式通过 CPU 的快速切换来让应用以为自己独自 CPU 。

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


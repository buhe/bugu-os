### 特权级

目标是这次也打印 Hello OS 不同的是，在用户态运行的应用。

为什么需要系统调用？特权？trap？CPU 虚拟化？直接都给权限，都在一个态不香么？试想一个应用就能破坏操作系统，操作系统多脆弱啊。应用有意无意的错误不要影响到操作系统和其他应用，这就需要硬件和操作系统配合来提供特权。应用只能运行在用户态，这样应用就可以放心使用 CPU ，这是 CPU 第一种虚拟化。

应用在用户态，操作系统在内核态。应用不能自己执行危险操作，想要执行必须通过系统调用委托给操作系统，而我们信赖操作系统。今天我们尝试在 k210 上实现特权级。

### 应用

像之前的内核一样，不同的是 lib.rs 内容是

```rust
#![no_std]
#![feature(asm)]
#![feature(linkage)]

use scall_os::{sys_exit, sys_write};

#[macro_use]
pub mod console;
mod lang;
mod scall_os;

#[no_mangle]
#[link_section = ".text.entry2"]
pub extern "C" fn _start() -> ! {
    clear_bss();
    exit(main());
    panic!("unreachable after sys_exit!");
}

#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
    panic!("Cannot find main!");
}

fn clear_bss() {
    extern "C" {
        fn start_bss();
        fn end_bss();
    }
    (start_bss as usize..end_bss as usize).for_each(|addr| {
        unsafe { (addr as *mut u8).write_volatile(0); }
    });
}


pub fn write(fd: usize, buf: &[u8]) -> isize { sys_write(fd, buf) }
pub fn exit(exit_code: i32) -> isize { sys_exit(exit_code) }
```

可以看见入口是 _start ，先给全局未初始化的变量都赋值 0 ，然后调用 main 函数，最后系统调用 OS 退出应用。注意 main 函数是 weak 的，链接的时候如果有 strong 就用 strong ，而他在 user/bin/hello.rs

```rust
#![no_std]
#![no_main]
#![feature(llvm_asm)]

#[macro_use]
extern crate user;

#[no_mangle]
fn main() -> i32 {
    println!("Hello OS");
    0
}
```

根据 rust 的规范，bin 下的文件会独立构建成可执行文件

### trap

trap 是不是很耳熟，应用要打印就会用 ecall 触发 trap ，我们先来实现 trap 。

先来看 trap 的初始化，trap/mod.rs 的内容

```rust
pub fn init() {
    extern "C" {
        fn __alltraps();
    }
    unsafe {
        stvec::write(__alltraps as usize, TrapMode::Direct);
    }
}
```

这个 init 函数在 main.rs 调用。__alltraps 是什么呢？trap/trap.asm 的内容

```asm
.altmacro
.macro SAVE_GP n
    sd x\n, \n*8(sp)
.endm
.macro LOAD_GP n
    ld x\n, \n*8(sp)
.endm
    .section .text
    .globl __alltraps
    .globl __restore
    .align 2
__alltraps:
    csrrw sp, sscratch, sp
    # now sp->kernel stack, sscratch->user stack
    # allocate a TrapContext on kernel stack
    addi sp, sp, -34*8
    # save general-purpose registers
    sd x1, 1*8(sp)
    # skip sp(x2), we will save it later
    sd x3, 3*8(sp)
    # skip tp(x4), application does not use it
    # save x5~x31
    .set n, 5
    .rept 27
        SAVE_GP %n
        .set n, n+1
    .endr
    # we can use t0/t1/t2 freely, because they were saved on kernel stack
    csrr t0, sstatus
    csrr t1, sepc
    sd t0, 32*8(sp)
    sd t1, 33*8(sp)
    # read user stack from sscratch and save it on the kernel stack
    csrr t2, sscratch
    sd t2, 2*8(sp)
    # set input argument of trap_handler(cx: &mut TrapContext)
    mv a0, sp
    call trap_handler

__restore:
    # case1: start running app by __restore
    # case2: back to U after handling trap
    mv sp, a0
    # now sp->kernel stack(after allocated), sscratch->user stack
    # restore sstatus/sepc
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    ld t2, 2*8(sp)
    csrw sstatus, t0
    csrw sepc, t1
    csrw sscratch, t2
    # restore general-purpuse registers except sp/tp
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr
    # release TrapContext on kernel stack
    addi sp, sp, 34*8
    # now sp->kernel stack, sscratch->user stack
    csrrw sp, sscratch, sp
    sret

```

__alltraps 都干了什么呢？应用运行要使用寄存器，于是系统调用前就保存寄存器、之后再恢复。sepc 寄存器比较特殊，trap 之后由它来决定后面执行什么。我们也可以利用它来执行第一个应用，一会就看见了。alltraps 最后  call trap_handler ，trap_handler 最后根据 system call id 来决定到底该调用哪个系统调用，这里用的 linux 的系统调用规范，当然也可以用别的，约定好就行。

我们再看看 trap/mod.rs 的最后的内容，包含 trap_handler

```rust
use riscv::register::{scause::{self, Exception, Trap}, stval, stvec, utvec::TrapMode};
use trap_ctx::TrapContext;

use crate::scall_sbi::syscall;

mod trap_ctx;
global_asm!(include_str!("trap.asm"));

pub fn init() {
    extern "C" {
        fn __alltraps();
    }
    unsafe {
        stvec::write(__alltraps as usize, TrapMode::Direct);
    }
}

#[no_mangle]
pub fn trap_handler(cx: &mut TrapContext) -> &mut TrapContext {
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            cx.sepc += 4;
            cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize;
        }
        Trap::Exception(Exception::StoreFault) | Trap::Exception(Exception::StorePageFault) => {
            println!("[kernel] PageFault in application, core dumped.");
            panic!(
                "StoreFault!"
            );
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            println!("[kernel] IllegalInstruction in application, core dumped.");
            panic!(
                "IllegalInstruction!"
            );
        }
        _ => {
            panic!(
                "Unsupported trap {:?}, stval = {:#x}!",
                scause.cause(),
                stval
            );
        }
    }
    cx
}
```

### task

那应用第一怎么执行的呢？

## 

（只有一个 app 可以 trap）
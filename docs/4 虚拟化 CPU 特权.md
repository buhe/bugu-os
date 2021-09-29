### 特权级

目标是这次也打印 Hello OS 不同的是，在用户态。

背景是，试想一个应用就能破坏操作系统，操作系统多脆弱啊。应用有意无意的错误不要影响到操作系统和其他应用，这就需要硬件和操作系统配合来提供特权。应用只能运行在用户态，这样应用就可以放心使用 CPU ，这是 CPU 第一种虚拟化。

应用在用户态，操作系统在内核态。应用不能自己执行危险操作，想要执行必须通过系统调用委托给操作系统，而我们信赖操作系统。今天我们尝试在 k210 上实现特权级。

### 应用



### trap

trap 是不是很耳熟，想要特权级切换就要 trap ，我们先来实现 trap 。

那么怎么 trap 呢？就要靠系统调用来触发，应用运行要使用寄存器，于是系统调用前就保存寄存器、之后再恢复，建立 trap/trap.asm，内容是

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



（只有一个 app 可以 trap）
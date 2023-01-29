.altmacro
.macro SAVE_GP n
    sd x\n, \n*8(sp)
.endm
.macro LOAD_GP n
    ld x\n, \n*8(sp)
.endm
    .section .text.trampoline
    .globl __alltraps
    .globl __restore
    .align 2
__alltraps:
    # __alltraps之前：sp指向用户空间的栈，sscratch指向了trap_ctx
    csrrw sp, sscratch, sp
    # 此时sp指向Trap上下文地址，sscratch指向用户栈
    # 保存 x0 ~ x31
    sd x1, 1*8(sp)
    # 跳过x2，x4
    sd x3, 3*8(sp)
    # 保存x5~x31
    .set n, 5
    .rept 27
        SAVE_GP %n
        .set n, n+1
    .endr
    # 保存sstatus和sepc
    # sepc和sstatus的值由CPU在Trap发生时自动生成
    csrr t0, sstatus
    csrr t1, sepc
    sd t0, 32*8(sp)
    sd t1, 33*8(sp)
    # 保存用户栈的sp到TrapContext的x2寄存器位置
    csrr t2, sscratch
    sd t2, 2*8(sp)
    # 加载内核的satp加载到t0
    ld t0, 34*8(sp)
    # 加载trap_handler地址到t1寄存器
    ld t1, 36*8(sp)
    # move to kernel_sp
    ld sp, 35*8(sp)
    # 修改satp寄存器为内核的satp
    csrw satp, t0
    sfence.vma
    # 跳转到trap_handler
    jr t1

__restore:
    # a0: TrapContext地址; a1: 用户空间的satp
    # 修改satp，切换到用户内存空间
    csrw satp, a1
    # 刷新TLB
    sfence.vma
    # 使sscratch指向TrapContext
    csrw sscratch, a0
    mv sp, a0
    # 此时sp指向用户空间的TrapContext
    # 恢复sepc和sstatus
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    csrw sstatus, t0
    csrw sepc, t1
    # 恢复x1、x3和x5~x31
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr
    # 将x2加载到sp，是sp指向用户空间的用户栈
    ld sp, 2*8(sp)
    sret
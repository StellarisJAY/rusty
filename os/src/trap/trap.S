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
# 保存Trap上下文到内核栈
__alltraps:
    # 此时sp指向用户栈，sscratch指向了内核栈
    # 经过此命令后，sp指向内核栈，sscratch指向用户栈
    csrrw sp, sscratch, sp
    # 在内核栈上分配34 * 8字节的栈帧，此时sp指向栈帧的底部
    addi sp, sp, -34*8
    # 保存x1寄存器到sp + 8，x0~x31分别保存到内核栈：sp + 8 * n位置
    sd x1, 1*8(sp)
    # skip sp(x2), we will save it later
    sd x3, 3*8(sp)
    # skip tp(x4), application does not use it
    # 循环，保存x5~x31寄存器
    .set n, 5
    .rept 27
        SAVE_GP %n
        .set n, n+1
    .endr
    # we can use t0/t1/t2 freely, because they were saved on kernel stack
    csrr t0, sstatus
    csrr t1, sepc
    // sstatus保存到 sp + 32 * 8
    sd t0, 32*8(sp)
    // sepc保存到 sp + 33 * 8
    sd t1, 33*8(sp)
    # 保存sscratch，即用户栈sp，到内核栈 sp + 16位置
    csrr t2, sscratch
    sd t2, 2*8(sp)
    # 将内核栈的sp交给TrapContext(cx: &mut TrapContext)，调用trap_handler
    mv a0, sp
    call trap_handler

__restore:
    # case1: start running app by __restore
    # case2: back to U after handling trap
    # 恢复的两种情况：
    # 启动App 或者 从S回到U
    mv sp, a0
    # 此时的sp是压入context后的内核栈顶
    # 恢复sstatus和sepc，两者分别保存在：sp + 8 * 32 和 sp + 8 * 33
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    ld t2, 2*8(sp)
    csrw sstatus, t0
    csrw sepc, t1
    # 从内核栈的x[2]读取sscratch，恢复sscratch
    # 因为在初始化context时，x[2]被赋值了用户栈sp
    # 所以经过该命令后，sscratch就成了用户栈sp
    csrw sscratch, t2
    # 恢复x0~x31
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr
    # 释放内核栈，将sp增加一个栈帧的大小
    addi sp, sp, 34*8
    # 交换sp和sscratch，使sp指向用户栈，sscratch指向内核栈
    csrrw sp, sscratch, sp
    # sret从S回到U
    sret
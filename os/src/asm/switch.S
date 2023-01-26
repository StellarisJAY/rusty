.altmacro
.macro SAVE_SN n
    sd s\n, (\n+2)*8(a0)
.endm
.macro LOAD_SN n
    ld s\n, (\n+2)*8(a1)
.endm
    .section .text
    .globl __switch
# switch 切换到下一个任务，需要做的事情：
# 保存当前任务的上下文：s0~s11寄存器、ra寄存器、内核栈的sp
# 加载下一个任务的上下文
# switch ret 会跳到ra所指示的位置，也就是restore程序
# restore程序的sp将是下一个任务的内核栈顶，它将从内核栈恢复U状态的寄存器，并回到U状态执行任务
# ra寄存器在switch过程中没有改变过，所以ra寄存器一直是__restore，__switch ret后都会通过__restore切换回U状态
__switch:
    # a0: current_ctx, a1: next_ctx
    # 保存sp寄存器到current_ctx
    sd sp, 8(a0)
    # 保存ra寄存器
    sd ra, 0(a0)
    # 保存s0~s11寄存器
    .set n, 0
    .rept 12
        SAVE_SN %n
        .set n, n + 1
    .endr
    # 读取下一个任务的ra寄存器值，加载到ra中
    ld ra, 0(a1)
    # 读取下一个任务的s0~s11
    .set n, 0
    .rept 12
        LOAD_SN %n
        .set n, n + 1
    .endr
    # 恢复下一个任务的内核栈的sp
    ld sp, 8(a1)
    # __switch流程结束，ret会返回到ra寄存器的地址
    ret
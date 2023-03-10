.altmacro
.macro SAVE_SN n
    sd s\n, (\n+2)*8(a0)
.endm
.macro LOAD_SN n
    ld s\n, (\n+2)*8(a1)
.endm
    .section .text
    .globl __switch

# 上下文中的ra寄存器指向了trap_return，通过该函数从S切换回U
# sp寄存器指向内核空间的栈
__switch:
    # a0: current_ctx, a1: next_ctx
    # 保存当前任务的内核地址空间的栈地址
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
    # 加载下一个任务的内核地址空间栈地址
    ld sp, 8(a1)
    # __switch流程结束，ret会返回到ra寄存器的地址，即跳到trap_return函数
    # trap_return地址是内核空间的虚地址，此时的satp指向内核pageTble，MMU自动完成地址转换
    ret
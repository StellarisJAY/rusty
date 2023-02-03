use crate::trap::trap_return;

// 进程上下文，用于在切换进程时保存通用寄存器
#[derive(Clone, Copy)]
#[repr(C)]
pub struct ProcessContext {
    pub ra: usize, // ra寄存器，__switch切换到该进程后的跳转地址
    pub sp: usize, // sp寄存器，内核栈指针
    pub s: [usize; 12], // 通用s0~s11寄存器
}

impl ProcessContext {
    pub fn new_empty_ctx() -> Self {
        Self { ra: 0, sp: 0, s: [0; 12] }
    }
    // 创建从trap_return返回用户空间的上下文，ra地址为trap_return函数，使switch后跳到trap返回逻辑
    pub fn trap_return_context(kernel_stack_sp: usize) -> Self {
        return Self { ra: trap_return as usize, sp: kernel_stack_sp, s: [0;12] };
    }
}
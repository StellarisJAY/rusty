use crate::trap::trap_return;
// 任务上下文，保存任务切换的通用寄存器
// ra寄存器：任务结束的
// sp寄存器：保存任务的栈指针
// s0~s11寄存器：任务执行过程中的通用寄存器
#[derive(Clone, Copy)]
#[repr(C)]
pub struct TaskContext {
    ra: usize,
    sp: usize,
    s: [usize; 12],
}

impl TaskContext {
    pub fn new_empty_ctx() -> Self {
        Self { ra: 0, sp: 0, s: [0; 12] }
    }

    pub fn trap_return_context(kernel_stack_sp: usize) -> Self {
        return Self { ra: trap_return as usize, sp: kernel_stack_sp, s: [0;12] };
    }
}
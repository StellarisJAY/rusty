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
    // 创建一个任务初始的通过trap _restore返回U状态的上下文
    pub fn trap_restore_context(kernel_stack_sp: usize) -> Self {
        extern "C" {
            fn __restore();
        }
        // 任务上下文的初始ra指向__restore程序，__switch会跳到restore，通过Trap restore切换到U状态
        // 同时此时的sp指向了内核栈，就不需要向__restore再传入a0来作为内核栈地址，由__switch程序设置sp
        TaskContext { ra: __restore as usize, sp: kernel_stack_sp, s: [0; 12] }
    }
}
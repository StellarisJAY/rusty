#[derive(Clone, Copy)]
#[repr(C)]
pub struct TaskContext {
    ra: usize,
    sp: usize,
    s: [usize; 12],
}

extern "C" {
    fn __restore();
}

impl TaskContext {
    pub fn new_empty_ctx() -> Self {
        Self { ra: 0, sp: 0, s: [0; 12] }
    }
    pub fn restore_ctx(kernel_stack_sp: usize) -> Self {
        // 任务上下文的初始ra指向__restore程序，__switch会跳到restore，通过Trap restore切换到U状态
        // 同时此时的sp指向了内核栈，就不需要向__restore再传入a0来作为内核栈地址，由__switch程序设置sp
        TaskContext { ra: __restore as usize, sp: kernel_stack_sp, s: [0; 12] }
    }
}
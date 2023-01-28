use riscv::register::sstatus;
use riscv::register::sstatus::{SPP,Sstatus};
#[repr(C)]
pub struct TrapContext {
    //x0~x31寄存器
    pub x: [usize;32],
    pub sstatus: Sstatus,
    // trap结束后跳转到的命令地址
    pub sepc: usize,
    // satp寄存器值，即该拥有该上下文的任务的页表ppn
    pub kernel_satp: usize,
    // 内核栈栈顶虚拟地址
    pub kernel_sp: usize,
    // trap处理器入口虚拟地址
    pub trap_handler: usize,
}

impl TrapContext {
    pub fn set_sp(&mut self, sp: usize) {
        // sp被保存到x2寄存器
        self.x[2] = sp;
    }
    
    pub fn task_init_context(app_entry: usize, user_sp: usize, kernel_sp: usize, kernel_satp: usize, trap_handler: usize) -> Self {
        let mut sstatus = sstatus::read();
        sstatus.set_spp(SPP::User);
        let mut ctx = TrapContext{
            x: [0; 32],
            sstatus: sstatus,
            sepc: app_entry, //离开S后跳回app_entry执行app
            kernel_satp: kernel_satp,
            kernel_sp: kernel_sp,
            trap_handler: trap_handler,
        };
        // 将ctx的sp设置成此时用户栈的sp
        // 使restore操作时的sscratch是用户栈sp
        ctx.set_sp(user_sp);
        return ctx;
    }
}
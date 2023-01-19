use riscv::register::sstatus;
use riscv::register::sstatus::{SPP,Sstatus};
#[repr(C)]
pub struct TrapContext {
    //x0~x31寄存器
    pub x: [usize;32],
    pub sstatus: Sstatus,
    // trap结束后跳转到的命令地址
    pub sepc: usize,
}

impl TrapContext {
    pub fn set_sp(&mut self, sp: usize) {
        // sp被保存到x2寄存器
        self.x[2] = sp;
    }
    // 初始化context，传入的是app的入口地址和用户栈的sp
    pub fn init_context(app_entry: usize, sp: usize) -> Self {
        let mut sstatus = sstatus::read();
        sstatus.set_spp(SPP::User);
        let mut ctx = TrapContext{
            x: [0; 32],
            sstatus: sstatus,
            sepc: app_entry //离开S后跳回app_entry执行app
        };
        // 将ctx的sp设置成此时用户栈的sp
        // 使restore操作时的sscratch是用户栈sp
        ctx.set_sp(sp);
        return ctx;
    }
}
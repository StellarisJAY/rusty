use crate::trap::context::TrapContext;
use crate::loader::APP_MANAGER;
use crate::loader;
use crate::config::{KERNEL_STACK_SIZE, USER_STACK_SIZE};

#[repr(align(4096))]
pub struct KernelStack {
    data: [u8;USER_STACK_SIZE]
}
#[repr(align(4096))]
pub struct UserStack {
    data: [u8;KERNEL_STACK_SIZE]
}

pub static KERNEL_STACK: KernelStack = KernelStack{data: [0;KERNEL_STACK_SIZE]};
pub static USER_STACK: UserStack = UserStack{data: [0; USER_STACK_SIZE]};

impl UserStack {
    pub fn get_sp(&self)->usize {
        // 栈顶地址就是数组的末尾地址
        return self.data.as_ptr() as usize + USER_STACK_SIZE;
    }
}

impl KernelStack {
    pub fn get_sp(&self) -> usize {
        return self.data.as_ptr() as usize + KERNEL_STACK_SIZE;
    }
    pub fn push_context(&self, cx: TrapContext) -> &'static mut TrapContext {
        let cx_ptr = (self.get_sp() - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
        unsafe {
            *cx_ptr = cx;
        }
        unsafe { cx_ptr.as_mut().unwrap() }
    }
}

// 运行app的本质就是从S回到U
// 所以本质上就是使用_restore汇编来恢复app需要的寄存器
// 此时压入内核栈的TrapContext实际上包含的是应用程序所需要恢复的寄存器
// context中的x[2]是用户栈的sp，sepc是app入口，sret之后就会到sepc执行app
pub fn run_next_app() -> ! {
    let mut app_manager = APP_MANAGER.exclusive_borrow();
    let current_app_id = app_manager.get_current_app();
    let num_apps = app_manager.get_num_apps();
    app_manager.move_to_next_app();
    drop(app_manager);
    if current_app_id >= num_apps {
        panic!("all app finished");
    }
    let app_base_addr = loader::get_base_addr(current_app_id);
    kernel_info!("running app_{}, base addr: {:#x}", current_app_id, app_base_addr);
    extern "C" {
        fn __restore(ctx_addr: usize);
    }
    let context = TrapContext::init_context(app_base_addr, USER_STACK.get_sp());
    unsafe {__restore(KERNEL_STACK.push_context(context) as *const _ as usize);}
    panic!("unreacheable after run app");
}

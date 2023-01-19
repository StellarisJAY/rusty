use lazy_static::lazy_static;
use crate::sync::UPSafeCell;
use core::arch::asm;
use crate::trap::context::TrapContext;

const MAX_APP_NUM: usize = 10;
const APP_BASE_ADDRESS: usize = 0x80400000;
const APP_SIZE_LIMIT: usize = 1024;
// 用户栈和内核栈分别为8KiB
const USER_STACK_SIZE: usize = 8 * 1024;
const KERNEL_STACK_SIZE: usize = 8 * 1024;

#[repr(align(4096))]
pub struct KernelStack {
    data: [u8;USER_STACK_SIZE]
}
#[repr(align(4096))]
pub struct UserStack {
    data: [u8;KERNEL_STACK_SIZE]
}

pub struct AppManager {
    num_apps: usize,
    current_app: usize,
    app_addrs: [usize; MAX_APP_NUM+1] // app基址数组
}

pub static KERNEL_STACK: KernelStack = KernelStack{data: [0;KERNEL_STACK_SIZE]};
pub static USER_STACK: UserStack = UserStack{data: [0; USER_STACK_SIZE]};

lazy_static! {
    pub static ref APP_MANAGER: UPSafeCell<AppManager> = unsafe {UPSafeCell::new({
        extern "C" {
            fn _num_app();
        }
        // app数组起始地址的指针
        let ptr = _num_app as usize as *const usize;
        // 数组第一个元素是app数量
        let num_apps: usize = ptr.read_volatile();
        let mut app_addrs = [0; MAX_APP_NUM + 1];
        // 从app数组读取所有app的基址，最后一项是最后一个app的结尾地址
        // 每个app的地址范围 = [app[i],app[i+1]）
        let raw_app_addrs: &[usize] = core::slice::from_raw_parts(ptr.add(1), num_apps + 1);
        app_addrs[..=num_apps].copy_from_slice(raw_app_addrs);
        AppManager{num_apps: num_apps, current_app: 0, app_addrs: app_addrs}
    })};
}

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


impl AppManager {
    pub fn get_current_app(&self) -> usize {
        return self.current_app;
    }
    pub fn get_num_apps(&self) -> usize {
        return self.num_apps;
    }
    pub fn get_app_addr(&self, id: usize) -> usize {
        return self.app_addrs[id];
    }
    pub fn move_to_next_app(&mut self) {
        self.current_app += 1;
    }
    // 加载一个app到指定的内存位置
    // 将操作系统内核里面链接的应用程序代码，加载到一个固定的内存区域
    // APP被加载到BASE_ADDR
    pub unsafe fn load_app(&self, id: usize) {
        if id >= self.num_apps {
            panic!("invalid app id");
        }
        info!("[kernel] loading app_{}", id);
        // 清空APP载入内存区域
        core::slice::from_raw_parts_mut(APP_BASE_ADDRESS as *mut u8, APP_SIZE_LIMIT)
        .fill(0);
        let app_size = self.app_addrs[id + 1] - self.app_addrs[id];
        // 加载app数据到切片中
        let app_data = core::slice::from_raw_parts(self.app_addrs[id] as *const u8, app_size);
        // 从APP_BASE_ADDR分配一块空闲区域
        let app_dst = core::slice::from_raw_parts_mut(APP_BASE_ADDRESS as *mut u8, app_size);
        // 将app数据拷贝到加载区域
        app_dst.copy_from_slice(app_data);
        // 执行fence指令，清空CPU缓存
        asm!("fence.i");
    }
}

// 运行app的本质就是从S回到U
// 所以本质上就是使用_restore汇编来恢复app需要的寄存器
// 此时压入内核栈的TrapContext实际上包含的是应用程序所需要恢复的寄存器
// context中的x[2]是用户栈的sp，sepc是app入口，sret之后就会到sepc执行app
pub fn run_next_app() -> ! {
    let mut app_manager = APP_MANAGER.exclusive_borrow();
    let current_app_id = app_manager.get_current_app();
    unsafe {app_manager.load_app(current_app_id);}
    app_manager.move_to_next_app();
    drop(app_manager);
    extern "C" {
        fn __restore(ctx_addr: usize);
    }
    let context = TrapContext::init_context(APP_BASE_ADDRESS, USER_STACK.get_sp());
    unsafe {__restore(KERNEL_STACK.push_context(context) as *const _ as usize);}
    panic!("unreacheable after run app");
}

pub fn run_app(app_id: usize) -> ! {
    let app_manager = APP_MANAGER.exclusive_borrow();
    unsafe {app_manager.load_app(app_id);}
    drop(app_manager);
    extern "C" {
        fn __restore(ctx_addr: usize);
    }
    let context = TrapContext::init_context(APP_BASE_ADDRESS, USER_STACK.get_sp());
    debug!("user stack sp: {:#x}, sepc: {:#x}, sstatus: {:?}", USER_STACK.get_sp(), context.sepc, context.sstatus);
    unsafe {__restore(KERNEL_STACK.push_context(context) as *const _ as usize);}
    panic!("unreacheable after run app");
}
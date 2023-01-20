use core::arch::asm;
use crate::config::{APP_BASE_ADDR, APP_SIZE_LIMIT};
use lazy_static::lazy_static;
use crate::sync::UPSafeCell;
use crate::config::MAX_APP_COUNT;

pub struct AppManager {
    num_apps: usize,
    current_app: usize,
    app_addrs: [usize; MAX_APP_COUNT]
}

impl AppManager {
    pub fn get_current_app(&self) -> usize {
        return self.current_app;
    }
    pub fn get_num_apps(&self) -> usize {
        return self.num_apps;
    }
    pub fn move_to_next_app(&mut self) {
        self.current_app += 1;
    }
}

lazy_static! {
    // AppManager初始化
    // 读取app数量，以及每个app所在的内存起始位置
    pub static ref APP_MANAGER: UPSafeCell<AppManager> = unsafe {UPSafeCell::new({
        extern "C" {
            fn _num_app();
        }
        // app数组起始地址的指针
        let ptr = _num_app as usize as *const usize;
        // 数组第一个元素是app数量
        let num_apps: usize = ptr.read_volatile();
        let app_addrs_raw = core::slice::from_raw_parts(ptr.add(1) as *const usize, num_apps + 1);
        let mut app_addrs: [usize; MAX_APP_COUNT] = [0; MAX_APP_COUNT];
        app_addrs[0..=num_apps].copy_from_slice(app_addrs_raw);
        AppManager{num_apps: num_apps, current_app: 0, app_addrs: app_addrs}
    })};
}


// 内核启动时，加载所有app到指定的内存区域
pub unsafe fn load_apps() {
    extern "C" {
        fn _num_app();
    }
    let app_manager = APP_MANAGER.exclusive_borrow();
    let num_apps = app_manager.get_num_apps();
    let app_start_addrs = app_manager.app_addrs;
    drop(app_manager);
    unsafe { asm!("fence.i"); }
    for id in 0..num_apps {
        // 计算app被加载到内核后的基地址
        let base_load_addr = get_base_addr(id);
        // 清空加载app的dst内存区域
        core::slice::from_raw_parts_mut(base_load_addr as *mut u8, APP_SIZE_LIMIT).fill(0);
        // 数组下一项的地址减去当前地址 = app大小
        let app_size = app_start_addrs[id + 1] - app_start_addrs[id];
        // 准备app在内核的目标区域 和 app代码源
        let app_dst = core::slice::from_raw_parts_mut(base_load_addr as *mut u8, app_size);
        let app_src = core::slice::from_raw_parts(app_start_addrs[id] as *const u8, app_size);
        // 拷贝app代码到内核app加载区域
        app_dst.copy_from_slice(app_src);
        kernel_info!("loaded app_{}, from: {:#x}, to: [{:#x}, {:#x}), app size: {} B", id,
        app_start_addrs[id],
        base_load_addr,
        base_load_addr + APP_SIZE_LIMIT,
        app_size);
    }
}

pub fn get_base_addr(app_id: usize) -> usize {
    return APP_BASE_ADDR + app_id * APP_SIZE_LIMIT;
}
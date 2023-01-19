const MAX_APP_NUM: usize = 10;
const APP_BASE_ADDRESS: usize = 0x80400000;
const APP_SIZE_LIMIT: usize = 1024;
use lazy_static::lazy_static;
use crate::sync::UPSafeCell;
use core::arch::asm;
pub struct AppManager {
    num_apps: usize,
    current_app: usize,
    app_addrs: [usize; MAX_APP_NUM+1] // app基址数组
}

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

impl AppManager {
    pub fn _get_current_app(&self) -> usize {
        return self.current_app;
    }
    pub fn get_num_apps(&self) -> usize {
        return self.num_apps;
    }
    pub fn get_app_addr(&self, id: usize) -> usize {
        return self.app_addrs[id];
    }
    // 加载一个app到指定的内存位置
    // 将操作系统内核里面链接的应用程序代码，加载到一个固定的内存区域
    pub unsafe fn load_app(&self, id: usize) {
        if id >= self.num_apps {
            panic!("invalid app id");
        }
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
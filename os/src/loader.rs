use crate::mem::address::*;

pub fn get_num_apps() -> usize {
    extern "C" {
        fn _num_app();
    }
    let pa = PhysAddr::new(_num_app as usize);
    println!("{:#x}, {}", _num_app as usize,  pa.floor().0);
    unsafe { (_num_app as usize as *const usize).read_volatile() }
}

// 加载app数据，获得一个字节数组
pub fn load_app_data(app_id: usize) -> &'static [u8] {
    extern "C" {
        fn _num_app();
    }
    let ptr = _num_app as usize as *const usize;
    unsafe {
        let num_apps = ptr.read_volatile() as usize;
        let app_addrs = core::slice::from_raw_parts(ptr.add(1), num_apps);
        return core::slice::from_raw_parts(app_addrs[app_id] as *const u8, app_addrs[app_id + 1] - app_addrs[app_id]);
    }
}
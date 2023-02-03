use lazy_static::lazy_static;
use alloc::vec::Vec;

pub fn get_num_apps() -> usize {
    extern "C" {
        fn _num_app();
    }
    unsafe { (_num_app as usize as *const usize).read_volatile() }
}

// 加载app数据，获得一个字节数组
pub fn load_app_data(app_id: usize) -> &'static [u8] {
    extern "C" {
        fn _num_app();
    }
    let num_apps = get_num_apps();
    let ptr = _num_app as usize as *const usize;
    unsafe {
        let app_addrs = core::slice::from_raw_parts(ptr.add(1), num_apps + 1);
        return core::slice::from_raw_parts(app_addrs[app_id] as *const u8, app_addrs[app_id + 1] - app_addrs[app_id]);
    }
}

lazy_static! {
    pub static ref APP_NAMES: Vec<&'static str> = unsafe{
        extern "C" {
            fn _app_names();
        }
        let mut start_ptr = _app_names as usize as *mut u8;
        let num_apps = get_num_apps();
        let mut v: Vec<&'static str> = Vec::new();
        for _ in 0..num_apps {
            let mut ptr = start_ptr;
            let mut length: usize = 0;
            while ptr.read_volatile() != b'\0' {
                ptr = ptr.add(1);
                length += 1;
            }
            let slice = core::slice::from_raw_parts_mut(start_ptr, length);
            let name = core::str::from_utf8_mut(slice);
            v.push(name.unwrap());
            start_ptr = ptr.add(1);
        }
        return v;
    };
}

pub fn get_app_data_by_name(name: &str) -> Option<&'static [u8]> {
    let num_apps = get_num_apps();
    return (0..num_apps)
    .find(| &i | {APP_NAMES[i] == name})
    .map(|app_id| {load_app_data(app_id)});
}

pub fn list_apps() {
    for (app_id, app_name) in APP_NAMES.iter().enumerate() {
        println!("app_id: {}, name: {}", app_id, app_name);
    }
}
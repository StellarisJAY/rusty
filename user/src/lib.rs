#![no_std]
#![feature(linkage)]
#![feature(panic_info_message)]

mod syscall;
#[macro_use]
pub mod console;
pub mod lang_items;

// 将该函数link到.text.entry，即kernel内存的开始位置
#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start(){
    // 清空bss
    clear_bss();
    // 执行main，并用main返回的exit_code退出
    exit(main());
}

// 弱链接，如果存在多个相同名称的函数，弱链接的会被覆盖
// 这里的默认main会被bin目录下的应用程序main覆盖
#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
    panic!("cannot find main");
}

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|addr| unsafe {
        (addr as *mut u8).write_volatile(0);
    });
}

use syscall::*;
pub fn write(fd: usize, buf: &[u8]) -> isize {
    sys_write(fd, buf)
}

pub fn exit(exit_code: i32) -> isize {
    sys_exit(exit_code)
}
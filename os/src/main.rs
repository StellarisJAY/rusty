#![no_std]
#![no_main]
#![feature(panic_info_message)]

mod lang_items;
mod sbi;
#[macro_use]
mod console;
mod syscall;

mod batch;
mod sync;
mod trap;

use core::arch::global_asm;
// 让编译器将该汇编代码文件作为编入全局代码
// 因为此时是rust main.rs文件的第一行代码
// 所以在编译完成后，该汇编文件的内容将作为所有代码的第一行
// 因此，entry.asm中的内容将负责完成系统的启动
global_asm!(include_str!("entry.asm"));
// 载入app程序代码
global_asm!(include_str!("link_app.S"));

// entry.asm中完成启动后，通过call rust_main命令跳转到该函数中
#[no_mangle]
pub fn rust_main() {
    kernel_info!("bootloader done");
    // 清空bss段
    clear_bss();
    kernel_info!(".bss section cleared");
    kernel_info!("System started");
    info!("display memory layout: ");
    display_kernel_memory();
    display_linked_apps();
    // 初始化陷入
    unsafe {trap::init();}
    batch::run_next_app();
    // 通过SBI陷入机器层，完成关机操作
//    sbi::shutdown();
}

// 清空bss段
fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    } {
        let start = sbss as usize;
        let end = ebss as usize;
        let mut current = start;
        while current <= end {
            unsafe {
                // 将current作为地址，转换成指针后修改数据的值为0
                (current as *mut u8).write_volatile(0);
            }
            current += 1;
        }
    }
}

// 打印.text .bss .data .rodata段的地址
fn display_kernel_memory() {
    extern "C" {
        fn stext();
        fn etext();
        fn srodata();
        fn erodata();
        fn sdata();
        fn edata();
        fn sbss();
        fn ebss();
        fn boot_stack_lower_bound();
        fn boot_stack_top();
    } {
        info!(".text section: [{:#x}, {:#x})", stext as usize, etext as usize);
        info!(".rodata section: [{:#x}, {:#x})", srodata as usize, erodata as usize);
        info!(".data section: [{:#x}, {:#x})", sdata as usize, edata as usize);
        info!("boot stack: [{:#x}, {:#x}), stack size: {}KiB",
        boot_stack_lower_bound as usize,
        boot_stack_top as usize,
        (boot_stack_top as usize - boot_stack_lower_bound as usize) / 1024);

        info!(".bss section: [{:#x}, {:#x})", sbss as usize, ebss as usize);

    }
}
use crate::batch::APP_MANAGER;
fn display_linked_apps() {
    let app_manager = APP_MANAGER.exclusive_borrow();
    let num_apps = app_manager.get_num_apps();
    info!("linked app count: {}", app_manager.get_num_apps());
    for i in 0..num_apps {
        info!("app[{}], kernel space addr: {:#x}, size: {} KiB", i,
        app_manager.get_app_addr(i),
        (app_manager.get_app_addr(i + 1) - app_manager.get_app_addr(i)) / 1024);
    }
    drop(app_manager);
}
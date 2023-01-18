#![no_std]
#![no_main]
#![feature(panic_info_message)]

mod lang_items;
mod sbi;
#[macro_use]
mod console;

use core::arch::global_asm;

// entry point: entry.asm
global_asm!(include_str!("entry.asm"));

#[no_mangle]
pub fn rust_main() -> ! {
    clear_bss();
    display_kernel_memory();
    sbi::shutdown();
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
    } {
        info!(".text section: [{:#x}, {:#x})", stext as usize, etext as usize);
        info!(".rodata section: [{:#x}, {:#x})", srodata as usize, erodata as usize);
        info!(".data section: [{:#x}, {:#x})", sdata as usize, edata as usize);
        info!(".bss section: [{:#x}, {:#x})", sbss as usize, ebss as usize);
    }
}
#![allow(unused)]
// 声明RustSBI
// 通过SBI调用RISC-V的基本服务
const SBI_SET_TIMER: usize = 0;
const SBI_CONSOLE_PUTCHAR: usize = 1;
const SBI_CONSOLE_GETCHAR: usize = 2;
const SBI_CLEAR_IPI: usize = 3;
const SBI_SEND_IPI: usize = 4;
const SBI_REMOTE_FENCE_I: usize = 5;
const SBI_REMOTE_SFENCE_VMA: usize = 6;
const SBI_REMOTE_SFENCE_VMA_ASID: usize = 7;
const SBI_SHUTDOWN: usize = 8;

use core::arch::asm;
use crate::println;

// call RISCV SBI 使用汇编ecall命令，一共4个参数：命令代码和3个args
#[inline(always)]
fn call_sbi(cmd: usize, arg0: usize, arg1: usize, arg2: usize) -> usize {
    let mut result;
    unsafe {
        asm! {
            "ecall",
            inlateout("x10") arg0 => result,
            in("x11") arg1,
            in("x12") arg2,
            in("x17") cmd,
        };
    }
    return result;
}

pub fn console_put_char(c: usize) {
    call_sbi(SBI_CONSOLE_PUTCHAR, c, 0, 0);
}

pub fn shutdown() -> ! {
    call_sbi(SBI_SHUTDOWN, 0, 0, 0);
    panic!("system shutdown");
}

pub fn sleep(duration: u64) {

}
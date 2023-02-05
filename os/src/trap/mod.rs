use core::arch::{global_asm, asm};
use riscv::register::{stvec, scause, stval, sie};
use riscv::register::utvec::TrapMode;
use riscv::register::scause::Trap;
use riscv::register::scause::{Exception, Interrupt};
use crate::syscall::syscall;
use crate::timer;
use crate::config::{TRAMPOLINE, TRAP_CONTEXT};
use crate::proc::{suspend_current_and_run_next, current_proc_trap_context, exit_current_and_run_next, current_proc_satp};

// 导入Trap上下文切换的汇编
global_asm!(include_str!("../asm/trap.S"));

pub mod context;

pub fn enable_stimer() {
    unsafe {
        sie::set_stimer();
    }
}

// 设置U到S的trap入口
fn set_user_mode_trap_entry() {
    unsafe {
        // 将stvec寄存器值改为虚拟地址空间的TRAMPOLINE地址
        // TRAMPOLINE虚拟地址对应的物理地址就是__all_traps的起始地址
        stvec::write(TRAMPOLINE as usize, TrapMode::Direct);
    }
}

fn set_kernel_trap_entry() {
    unsafe {
        stvec::write(trap_from_kernel as usize, TrapMode::Direct);
    }
}

#[no_mangle]
pub fn trap_from_kernel() -> ! {
    panic!("trap from kernel not allowed");
}

#[no_mangle]
pub fn trap_return() -> ! {
    set_user_mode_trap_entry();
    let trap_cx_ptr = TRAP_CONTEXT;
    let user_satp = current_proc_satp();
    extern "C" {
        fn __alltraps();
        fn __restore();
    }
    // restore段的虚拟地址 = TRAMPOLINE虚拟地址 + __all_trap段大小
    let restore_va = __restore as usize - __alltraps as usize + TRAMPOLINE;
    // 通过jr跳转到restore虚拟地址，处理器自动完成地址转换
    unsafe {
        asm!("fence.i",
            "jr {restore_va}",
            restore_va = in(reg) restore_va,
            in("a0") trap_cx_ptr, // a0 寄存器写入trap上下文地址
            in("a1") user_satp,   // a1 寄存器写入用户地址空间的satp，即用户地址空间的页表ppn
            options(noreturn)
        );
    }
}


#[no_mangle]
pub fn trap_handler() {
    set_kernel_trap_entry();

    // 从scause和stval读取trap原因和trap信息
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        // 捕获到U传给S的ecall，转到syscall处理，在syscall模块对不合法的syscall过滤
        Trap::Exception(Exception::UserEnvCall) => {
            let mut ctx = current_proc_trap_context();
            // trap结束后的指令为trap指令之后的一条指令
            // 每条指令大小为4字节，所以sepc寄存器 + 4
            ctx.sepc += 4;
            // 完成系统调用，将trap之前的上下文中的x17,x10,x11,x12传入ecall
            let result = syscall(ctx.x[17], [ctx.x[10], ctx.x[11], ctx.x[12]]) as usize;
            ctx = current_proc_trap_context();
            ctx.x[10] = result;
        },
        // 时钟中断，任务时间片结束
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            // 重置时钟中断
            timer::set_next_time_trigger();
            // 切换到下一个任务
            suspend_current_and_run_next();
        },
        Trap::Exception(Exception::StoreFault) |
        Trap::Exception(Exception::StorePageFault) |
        Trap::Exception(Exception::InstructionFault) |
        Trap::Exception(Exception::InstructionPageFault) |
        Trap::Exception(Exception::LoadFault) |
        Trap::Exception(Exception::LoadPageFault) => {
            println!(
                "[kernel] {:?} in application, bad addr = {:#x}, bad instruction = {:#x}, core dumped.",
                scause.cause(),
                stval,
                current_proc_trap_context().sepc,
            );
            // page fault 退出，返回-2
            exit_current_and_run_next(-2);
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            println!("[kernel] IllegalInstruction in application, core dumped.");
            // 非法指令退出，返回-3
            exit_current_and_run_next(-3);
        }
        _ => {
            panic!("unsupported trap, cause: {:?}, stval: {:#x}", scause.cause(), stval);
        }
    }
    trap_return();
}
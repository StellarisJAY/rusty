use core::arch::global_asm;
use riscv::register::{stvec, scause, stval, sie};
use riscv::register::utvec::TrapMode;
use riscv::register::scause::Trap;
use riscv::register::scause::{Exception, Interrupt};
use crate::syscall::syscall;
use crate::task::{run_next_task, exit_current_task, suspend_current_task};
use crate::timer;
// 导入Trap上下文切换的汇编
global_asm!(include_str!("trap.S"));

pub mod context;

pub unsafe fn init() {
    extern "C" {
        fn __alltraps();
    }
    // 将all_trap汇编的地址写入stvec寄存器，即Trap处理入口寄存器
    // 之后发生trap后会从stvec寄存器找到trap处理逻辑
    stvec::write(__alltraps as usize, TrapMode::Direct);
}

pub fn enable_stimer() {
    unsafe {
        sie::set_stimer();
    }
}

use context::TrapContext;
#[no_mangle]
pub fn trap_handler(ctx: &mut TrapContext) -> &mut TrapContext {
    // 从scause和stval读取trap原因和trap信息
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        // 捕获到U传给S的ecall，转到syscall处理，在syscall模块对不合法的syscall过滤
        Trap::Exception(Exception::UserEnvCall) => {
            // trap结束后的指令为trap指令之后的一条指令
            // 每条指令大小为4字节，所以sepc寄存器 + 4
            ctx.sepc += 4;
            // 完成系统调用，将trap之前的上下文中的x17,x10,x11,x12传入ecall
            // ecall返回值传给x10
            ctx.x[10] = syscall(ctx.x[17], [ctx.x[10], ctx.x[11], ctx.x[12]]) as usize;
        },
        // 时钟中断，任务时间片结束
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            // 重置时钟中断
            timer::set_next_time_trigger();
            // 切换到下一个任务
            suspend_current_task();
            run_next_task();
        },
        Trap::Exception(Exception::StoreFault | Exception::StorePageFault) => {
            error!("[kernel] Page fault, kernel kills application");
            exit_current_task();
            run_next_task();
        },
        Trap::Exception(Exception::InstructionFault) => {
            error!("[kernel] Instruction Fault: {:#x}", stval);
            exit_current_task();
            run_next_task();
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            error!("[kernel] Illegal instruction, kernel kills application");
            exit_current_task();
            run_next_task();
        }
        _ => {
            panic!("unsupported trap, cause: {:?}, stval: {:#x}", scause.cause(), stval);
        }
    }
    return ctx;
}
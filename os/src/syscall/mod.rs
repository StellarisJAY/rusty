use core::arch::asm;

const SYS_EXIT: usize = 93;

fn syscall(id: usize, args: [usize;3]) -> isize {
    let mut result: isize;
    unsafe{
        asm!(
                "ecall",
        inlateout("x10") args[0] => result,
        in("x11") args[1],
        in("x12") args[2],
        in("x13") id
        );
    }
    return result;
}

pub fn sys_exit(exit_code: i32) -> isize {
    syscall(SYS_EXIT, [exit_code as usize, 0, 0])
}
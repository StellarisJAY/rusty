use core::arch::asm;

const SYS_WRITE: usize = 64;
const SYS_EXIT: usize = 93;

fn syscall(id: usize, args: [usize; 3]) -> isize {
    let mut result: isize;
    unsafe {
        asm!("ecall",
            inlateout("x10") args[0] => result,
            in("x11") args[1],
            in("x12") args[2],
            in("x17") id
        );
    }
    return result;
}

pub fn sys_write(fd: usize, buf: &[u8]) -> isize {
    syscall(SYS_WRITE, [fd, buf.as_ptr() as usize, buf.len()])
}

pub fn sys_exit(exit_code: i32)-> isize {
    syscall(SYS_EXIT, [exit_code as usize, 0, 0])
}

// os/src/syscall/mod.rs
pub mod fs;
pub mod proc;
use fs::*;
use proc::*;

const SYS_CALL_READ: usize = 63;
const SYS_CALL_WRITE: usize = 64;
const SYS_CALL_EXIT: usize = 93;
const SYS_CALL_YIELD: usize = 124;
const SYS_CALL_GET_TIME: usize = 169;

const SYS_CALL_FORK: usize = 220;
const SYS_CALL_WAITPID: usize = 260;
const SYS_CALL_EXEC: usize = 221;

pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    match syscall_id {
        SYS_CALL_READ => sys_read(args[0], args[1] as *mut u8, args[2]),
        SYS_CALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYS_CALL_EXIT => sys_exit(args[0] as i32),
        SYS_CALL_YIELD => sys_yield(),
        SYS_CALL_GET_TIME => sys_get_time(),
        SYS_CALL_FORK => sys_fork(),
        SYS_CALL_EXEC => sys_exec(args[0] as *const u8),
        SYS_CALL_WAITPID => sys_waitpid(args[0] as isize, args[1] as *mut i32),
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}
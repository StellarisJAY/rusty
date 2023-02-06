#![no_std]
#![no_main]

use lib_rusty::*;

#[no_mangle]
pub fn main() -> isize {
    if fork() == 0 {
        exec("user_shell\0");
    }else {
        loop {
            let mut exit_code: i32 = 0;
            let exit_pid = wait(&mut exit_code);
            if exit_pid == -1 {
                yield_();
                continue;
            }
            println!("init proc recycled a zombile proc, pid: {}, exit_code: {}", exit_pid, exit_code);
        }
    }
    return 0;
}
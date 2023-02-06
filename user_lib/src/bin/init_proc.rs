#![no_std]
#![no_main]

use lib_rusty::*;

#[no_mangle]
pub fn main() -> isize {
    let pid = spawn("user_shell\0");
    if pid == -1 {
        return -1;
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
}
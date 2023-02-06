#![no_std]
#![no_main]
extern crate alloc;

#[macro_use]
extern crate lib_rusty;
use lib_rusty::*;
use lib_rusty::console::{get_char};
use alloc::string::String;

const CR: u8 = b'\r';
const LF: u8 = b'\n';
const DL: u8 = 0x7f;
const BS: u8 = 0x08;

#[no_mangle]
pub fn main() -> isize {
    println!("User command line entered");
    let mut line = String::new();
    print!(">>> ");
    loop {
        let c = get_char();
        match c {
            CR | LF => {
                println!("");
                if !line.is_empty() {
                    line.push('\0');
                    let pid = spawn(&line);
                    if pid == -1 {
                        shell_error!("command \"{}\" not found", &line);
                        line.clear();
                    }else {
                        // 等待子进程结束
                        let mut exit_code: i32 = 0;
                        let exit_pid = waitpid(pid, &mut exit_code);
                        assert_eq!(pid, exit_pid);
                        line.clear();
                    }
                }
                print!(">>> ");
            },
            BS | DL => {
                print!("{}", BS as char);
                print!(" ");
                print!("{}", BS as char);
                line.pop();
            },
            _ => {
                print!("{}", c as char);
                line.push(c as char);
            }
        }
    }
}

#[macro_export]
macro_rules! shell_error {
    ($fmt: literal $(, $($arg: tt)+)?)=>{
        $crate::lib_rusty::console::print_str("\x1b[31m[shell] [ERROR] ");
        $crate::lib_rusty::console::print(format_args!($fmt $(, $($arg)+)?));
        $crate::lib_rusty::console::print_str("\x1b[0m\n");
    }
}
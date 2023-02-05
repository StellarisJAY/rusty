#![no_std]
#![no_main]
extern crate alloc;

#[macro_use]
extern crate lib_rusty;
use lib_rusty::{exec, fork, waitpid};
use lib_rusty::console::{get_char};
use alloc::string::String;

const CR: u8 = b'\r';
const LF: u8 = b'\n';
const DL: u8 = 0x7f;
const BS: u8 = 0x08;

#[no_mangle]
pub fn main() -> isize {
    let mut line = String::new();
    print!(">>> ");
    loop {
        let c = get_char();
        match c {
            CR | LF => {
                println!("");
                if !line.is_empty() {
                    line.push('\0');
                    let pid = fork();
                    if pid == 0 {
                        // 子进程
                         if exec(&line) == -1 {
                             println!("error when executing: {}", line);
                             return -4;
                         }
                    }else {
                        // shell进程
                        let mut exit_code: i32 = 0;
                        let exit_pid = waitpid(pid, &mut exit_code);
                        assert_eq!(pid, exit_pid);
                        println!("pid: {} exited with code: {}", pid, exit_code);
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
use core::fmt::{self, Write};
use crate::{sys_write, sys_read};
const STDOUT:usize = 1;
const STDIN: usize = 0;

struct Stdout;
// implement Write trait for Stdout
impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // 用户层的不能直接通过SBI打印
        // 必须通过ecall到内核层
        // 内核层再根据syscall_id决定是否ecall机器层
        sys_write(STDOUT, s.as_bytes());
        return Ok(());
    }
}

pub fn get_char() -> u8 {
    let mut buffer = [0u8; 1];
    sys_read(STDIN, &mut buffer);
    return buffer[0];
}

pub fn print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}
pub fn print_str(s: &str) {
    Stdout.write_str(s).unwrap();
}

#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! error {
    ($fmt: literal $(, $($arg: tt)+)?)=>{
        $crate::console::print_str("\x1b[31m[ERROR] ");
        $crate::console::print(format_args!($fmt $(, $($arg)+)?));
        $crate::console::print_str("\x1b[0m\n");
    }
}

#[macro_export]
macro_rules! info {
    ($fmt: literal $(, $($arg: tt)+)?)=>{
        $crate::console::print_str("[INFO] ");
        $crate::console::print(format_args!($fmt $(, $($arg)+)?));
        $crate::console::print_str("\n");
    }
}

#[macro_export]
macro_rules! warn {
    ($fmt: literal $(, $($arg: tt)+)?)=>{
        $crate::console::print_str("\x1b[93m[WARN] ");
        $crate::console::print(format_args!($fmt $(, $($arg)+)?));
        $crate::console::print_str("\x1b[0m\n");
    }
}

#[macro_export]
macro_rules! debug {
    ($fmt: literal $(, $($arg: tt)+)?)=>{
        $crate::console::print_str("\x1b[32m[DEBUG] ");
        $crate::console::print(format_args!($fmt $(, $($arg)+)?));
        $crate::console::print_str("\x1b[0m\n");
    }
}
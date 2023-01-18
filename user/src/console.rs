use core::fmt::{self, Write};
use crate::syscall;

const STDOUT:usize = 1;
struct Stdout;
// implement Write trait for Stdout
impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        syscall::sys_write(STDOUT, s.as_bytes());
        return Ok(());
    }
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
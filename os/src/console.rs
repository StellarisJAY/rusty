use core::fmt::{self, Write};
use crate::sbi;
struct StdOut;

// implement Write trait for Stdout
impl Write for StdOut {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            sbi::console_put_char(c as usize);
        }
        return Ok(());
    }
}

pub fn print(args: fmt::Arguments) {
    StdOut.write_fmt(args).unwrap();
}
pub fn print_str(s: &str) {
    StdOut.write_str(s).unwrap();
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
use crate::mem::page_table::translated_byte_buffer;
use crate::proc::{current_proc_satp, suspend_current_and_run_next};
use crate::sbi::console_get_char;


const FD_STDOUT: usize = 1;
const FD_STDIN: usize = 0;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            let buffers = translated_byte_buffer(current_proc_satp(), buf, len);
            for buffer in buffers {
                print!("{}", core::str::from_utf8(buffer).unwrap());
            }
            len as isize
        },
        _ => {
            panic!("Unsupported fd in sys_write!");
        }
    }
}

pub fn sys_read(fd: usize, buf: *mut u8, len: usize) -> isize {
    match fd {
        FD_STDIN=> {
            assert_eq!(len, 1, "currently only allow 1 byte per read");
            let mut c: usize;
            loop {
                c = console_get_char();
                if c == 0 {
                    suspend_current_and_run_next();
                    continue;
                }else {
                    break;
                }
            }
            let ch = c as u8;
            let mut buffers = translated_byte_buffer(current_proc_satp(), buf, len);
            unsafe {buffers[0].as_mut_ptr().write_volatile(ch);}
            return len as isize;
        },
        _ => {
            panic!("unsupported fd in sys_read");
        }
    }
}


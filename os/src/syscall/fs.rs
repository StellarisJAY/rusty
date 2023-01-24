const FD_STDOUT: usize = 1;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            let slice = unsafe { core::slice::from_raw_parts(buf, len) };
            if slice.len() == 0 {
                return 0;
            }
            match core::str::from_utf8(slice) {
                Ok(str) => {
                    print!("{}", str);
                    return len as isize;
                },
                Err(error) => {
                    error!("interpret slice as utf-8 string error: {}", error);
                    0
                },
            }
        },
        _ => {
            panic!("Unsupported fd in sys_write!");
        }
    }
}


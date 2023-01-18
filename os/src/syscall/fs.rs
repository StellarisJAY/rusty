const FD_STDOUT: usize = 0;

pub fn sys_write(fd: usize, buffer: &[u8]) -> isize {
    match fd {
        FD_STDOUT => {
            let slice = unsafe { core::slice::from_raw_parts(buf, len) };
            let str = core::str::from_utf8(slice).unwrap();
            print!("{}", str);
            len as isize
        },
        _ => panic!("unsupported fd"),
    }
}
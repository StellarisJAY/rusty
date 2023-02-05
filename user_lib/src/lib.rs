#![no_std]
#![feature(linkage)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

mod syscall;
#[macro_use]
pub mod console;
pub mod lang_items;
// 用户堆内存大小：4MiB
const USER_HEAP_SIZE: usize = 4096 * 1024;

use buddy_system_allocator::LockedHeap;

// 用户应用程序的堆内存分配
#[global_allocator]
static HEAP: LockedHeap = LockedHeap::new();
static mut HEAP_SPACE: [u8; USER_HEAP_SIZE] = [0; USER_HEAP_SIZE];

#[no_mangle]
#[alloc_error_handler]
pub fn heap_alloc_error_handler(layout: core::alloc::Layout) -> ! {
    panic!("alloc heap memory error: {:?}", layout);
}

// 将该函数link到.text.entry，即kernel内存的开始位置
#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start(){
    init_user_heap();
    // 执行main，并用main返回的exit_code退出
    exit(main());
}

// 弱链接，如果存在多个相同名称的函数，弱链接的会被覆盖
// 这里的默认main会被bin目录下的应用程序main覆盖
#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
    panic!("cannot find main");
}

fn init_user_heap() {
    unsafe {
        HEAP.lock().init(HEAP_SPACE.as_ptr() as usize, USER_HEAP_SIZE);
    }
}

use syscall::*;
pub fn write(fd: usize, buf: &[u8]) -> isize {
    sys_write(fd, buf)
}

pub fn exit(exit_code: i32) -> isize {
    sys_exit(exit_code)
}

pub fn yield_() -> isize {
    sys_yield()
}

pub fn get_time() -> isize {
    sys_get_time()
}

pub fn wait(exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(-1, exit_code as *mut _) {
            -2 => {yield_();},
            pid => return pid,
        }
    }
}

pub fn waitpid(pid: isize, exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(pid, exit_code as *mut _) {
            -2 => {yield_();},
            exit_pid => return exit_pid,
        }
    }
}

pub fn exec(path: &str)->isize {
    sys_exec(path)
}

pub fn fork() -> isize {
    sys_fork()
}

pub fn read(fd: usize, buffer: &mut [u8]) -> isize {
    sys_read(fd, buffer)
}
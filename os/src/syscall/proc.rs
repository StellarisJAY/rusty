use crate::task::{suspend_current_task, exit_current_task, run_next_task};
use crate::timer::get_time_ms;
use crate::proc::{suspend_current_and_run_next, current_process, add_process};

pub fn sys_exit(xstate: i32) -> ! {
    kernel_info!("Application exited with code {}", xstate);
    exit_current_task();
    run_next_task()
}

pub fn sys_yield() -> isize{
    debug!("application yield");
    suspend_current_and_run_next();
    0
}

pub fn sys_get_time() -> isize {
    get_time_ms() as isize
}

pub fn sys_fork() -> isize {
    let current_proc = current_process().unwrap();
    let new_proc = current_proc.fork();
    let pid = new_proc.pid.0;
    // 子进程的fork返回0
    new_proc.exclusive_borrow_inner().get_trap_context().x[10] = 0;
    add_process(new_proc);
    // 父进程的fork返回子进程的pid
    return pid as isize;
}
use crate::task::{suspend_current_task, exit_current_task, run_next_task};
use crate::timer::get_time_ms;
use crate::proc::{suspend_current_and_run_next, current_process, add_process, current_proc_satp};
use crate::mem::page_table::translate_string;
use crate::loader::get_app_data_by_name;

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

// fork系统调用，从当前进程fork子进程，向父进程返回子进程的pid，向子进程返回0
pub fn sys_fork() -> isize {
    // 从当前进程fork出子进程
    let current_proc = current_process().unwrap();
    let child_proc = current_proc.fork();
    let pid = child_proc.pid.0;
    // 子进程的fork返回0
    child_proc.exclusive_borrow_inner().get_trap_context().x[10] = 0;
    add_process(child_proc);
    // 父进程的fork返回子进程的pid
    return pid as isize;
}

// exec系统调用，加载指定path应用程序并执行
pub fn sys_exec(path: *const u8) -> isize {
    let satp = current_proc_satp();
    let path_str = translate_string(satp, path);
    if let Some(elf_data) = get_app_data_by_name(&path_str) {
        let proc = current_process().unwrap();
        proc.exec(elf_data);
        return 0;
    }else {
        return -1;
    }
}
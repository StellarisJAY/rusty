use crate::timer::get_time_ms;
use crate::proc::{suspend_current_and_run_next, current_process, add_process, current_proc_satp, exit_current_and_run_next};
use crate::proc::pcb::ProcessStatus;
use crate::mem::page_table::{translate_string, translate_ptr};
use crate::loader::get_app_data_by_name;

pub fn sys_exit(xstate: i32) -> ! {
    kernel_info!("Application exited with code {}", xstate);
    exit_current_and_run_next(xstate);
    panic!("unreachable");
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

// waitpid系统调用，等待子进程结束
// pid如果为-1，
pub fn sys_waitpid(pid: isize, exit_code: *mut i32) -> isize {
    let current_proc = current_process().unwrap();
    let mut inner = current_proc.exclusive_borrow_inner();
    // 查看pid是否是当前进程的子进程，不是则返回-1
    // 参数pid为-1，表示随机返回一个子进程
    if inner.children.iter().find(|proc|{pid == -1 || proc.get_pid() == pid as usize}).is_none() {
        drop(inner);
        return -1;
    }
    // 查看该进程是否是僵尸进程
    let pair = inner.children.iter()
    .enumerate()
    .find(|(_,proc)|{
        (proc.exclusive_borrow_inner().status == ProcessStatus::Zombie) && proc.get_pid() == pid as usize
    });

    if let Some((idx, _)) = pair {
        let child = inner.children.remove(idx);
        let child_inner = child.exclusive_borrow_inner();
        // 获取exit_code指针在父进程地址空间的物理地址，将子进程退出码赋值给指针
        *translate_ptr(inner.user_space_satp(), exit_code) = child_inner.exit_code;
        drop(child_inner);
        return pid as isize;
    }
    return -2;
}

// sys_wait，等待任意一个子进程结束，返回子进程pid
pub fn sys_wait(exit_code: *mut i32) -> isize {
    loop {
        match sys_waitpid(-1, exit_code) {
            -2 => {sys_yield();},
            exited_pid => {return exited_pid;}
        }
    }
}
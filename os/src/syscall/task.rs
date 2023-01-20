use crate::task::{suspend_current_task, exit_current_task, run_next_task};

pub fn sys_exit(xstate: i32) -> ! {
    kernel_info!("Application exited with code {}", xstate);
    exit_current_task();
    run_next_task()
}

pub fn sys_yield() -> isize{
    debug!("application yield");
    suspend_current_task();
    run_next_task();
}
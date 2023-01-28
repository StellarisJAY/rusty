use core::arch::global_asm;
use crate::sync::UPSafeCell;
use lazy_static::lazy_static;
use crate::loader::{get_num_apps, load_app_data};
use task::*;
use context::*;
use alloc::vec::Vec;

mod context;
pub mod task;

global_asm!(include_str!("../asm/switch.S"));

extern "C" {
    pub fn __switch(current_ctx: *mut TaskContext, next_ctx: *const TaskContext);
}

pub struct TaskManager {
    pub num_tasks: usize,
    instance: UPSafeCell<TaskManagerInstance>,
}

struct TaskManagerInstance {
    task_control_blocks: Vec<TaskControlBlock>,
    current_task: usize,
}

lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = {
        let num_apps = get_num_apps();
        // 创建空的task数组
        let mut tasks: Vec<TaskControlBlock> = Vec::new();

        for app_id in 0..num_apps {
            let tcb = TaskControlBlock::new(load_app_data(app_id), app_id);
            tcb.status = TaskStatus::Ready;
            tasks.push(tcb);
        }
        let instance = unsafe {
            UPSafeCell::new(TaskManagerInstance{
                    task_control_blocks: tasks,
                    current_task: 0,
            })
        };
        TaskManager { num_tasks: num_apps, instance:  instance}
    };
}

pub fn exit_current_task() {
    TASK_MANAGER.exit_current_task();
}

pub fn suspend_current_task() {
    TASK_MANAGER.suspend_current_task();
}

pub fn run_next_task() -> ! {
    TASK_MANAGER.run_next_task()
}

pub fn run_first_task() -> !{
    TASK_MANAGER.run_first_task()
}


impl TaskManager {
    fn suspend_current_task(& self) {
        let mut manager = self.instance.exclusive_borrow();
        let task_id = manager.current_task;
        manager.task_control_blocks[task_id].status = TaskStatus::Ready;
        drop(manager);
    }
    fn exit_current_task(& self) {
        let mut manager = self.instance.exclusive_borrow();
        let task_id = manager.current_task;
        manager.task_control_blocks[task_id].status = TaskStatus::Exited;
        drop(manager);
    }

    fn find_next_task(&self) -> Option<usize> {
        let manager = self.instance.exclusive_borrow();
        let tasks = manager.task_control_blocks;
        for id in 0..tasks.len() {
            if tasks[id].status == TaskStatus::Ready {
                drop(manager);
                return Some(id);
            }
        }
        drop(manager);
        return None;
    }

    fn run_first_task(&self) -> ! {
        let mut instance = self.instance.exclusive_borrow();
        let mut task0 = instance.task_control_blocks[0];
        instance.current_task = 0;
        task0.status = TaskStatus::Running;
        let mut _unused = TaskContext::new_empty_ctx();
        drop(instance);
        unsafe {
            __switch(&mut _unused as *mut TaskContext, & task0.ctx as *const TaskContext);
        }
        panic!("unreachable in run_first_task!");
    }

    fn run_next_task(&self) -> !{
        if let Some(next_id) = self.find_next_task() {
            let mut manager = self.instance.exclusive_borrow();
            let current_task = manager.current_task;
            let cur_ctx_ptr: *mut TaskContext;
            manager.task_control_blocks[next_id].status = TaskStatus::Running;
            manager.current_task = next_id;
            // 获取ctx的地址
            // 时间片结束的切换可能因为只剩下当前任务，所以并没有切换任务
            // 需要创建一个空白的context来作为当前context
            if next_id == current_task {
                let mut empty_ctx = TaskContext::new_empty_ctx();
                cur_ctx_ptr = &mut empty_ctx as *mut TaskContext;
            }else {
                cur_ctx_ptr = &mut manager.task_control_blocks[current_task].ctx as *mut TaskContext;
            }
            let next_ctx_ptr = & manager.task_control_blocks[next_id].ctx as *const TaskContext;
            drop(manager);
            debug!("running next task, app_{}", next_id);
            // switch会将下一个任务的sp、ra恢复，并通过restore回到U状态
            unsafe {
                __switch(cur_ctx_ptr, next_ctx_ptr);
            }

            panic!("unreachable");
        }else {
            panic!("all tasks finished");
        }
    }
}



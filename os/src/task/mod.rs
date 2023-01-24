use core::arch::global_asm;
use context::TaskContext;
use crate::sync::UPSafeCell;
use crate::config::MAX_TASK_COUNT;
use lazy_static::lazy_static;
use crate::loader::{get_num_apps, get_base_addr, USER_STACK, KERNEL_STACK};
use crate::trap::context::TrapContext;

mod context;

global_asm!(include_str!("switch.S"));

extern "C" {
    pub fn __switch(current_ctx: *mut TaskContext, next_ctx: *const TaskContext);
}

// 任务状态枚举
#[derive(Clone, Copy, PartialEq)]
pub enum TaskStatus {
    New, // 新建，未初始化
    Ready, // 就绪，已初始化，可运行
    Running, // 运行中
    Exited, // 已结束
}
// TaskControlBlock 任务控制块
// 保存当前任务的状态，以及任务的上下文
#[derive(Clone, Copy)]
pub struct TaskControlBlock {
    pub status: TaskStatus,
    pub ctx: TaskContext
}

pub struct TaskManager {
    pub num_tasks: usize,
    instance: UPSafeCell<TaskManagerInstance>,
}

struct TaskManagerInstance {
    task_control_blocks: [TaskControlBlock; MAX_TASK_COUNT],
    current_task: usize,
}

lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = {
        let num_apps = get_num_apps();
        // 创建空的task数组
        let mut tasks = [TaskControlBlock{
            status: TaskStatus::New,
            ctx:TaskContext::new_empty_ctx()
        };
        MAX_TASK_COUNT];

        for app_id in 0..num_apps {
            tasks[app_id].status = TaskStatus::Ready;
            // 为每一个任务的内核栈创建一个TrapContext，用来通过restore启动app
            // trap的sepc设置为app的入口地址，使restore程序能够跳转到app代码
            let trap_ctx = TrapContext::init_context(get_base_addr(app_id), USER_STACK[app_id].get_sp());
            // 创建一个ra指向__restore的任务上下文，当切换到该任务时，通过restore切换回U状态，执行任务
            tasks[app_id].ctx = TaskContext::restore_ctx(KERNEL_STACK[app_id].push_context(trap_ctx) as *const _ as usize);
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



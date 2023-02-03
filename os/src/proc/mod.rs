pub mod pid;
pub mod stack;
pub mod pcb;
pub mod context;

use lazy_static::lazy_static;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use crate::sync::UPSafeCell;
use pcb::*;
use context::*;
use crate::trap::context::TrapContext;
use crate::loader::get_app_data_by_name;

// 切换进程的汇编函数
extern "C" {
    pub fn __switch(cur_ctx: *mut ProcessContext, next_ctx: *const ProcessContext);
}

// 进程管理器
pub struct ProcessManager {
    queue: VecDeque<Arc<ProcessControlBlock>> // 可执行的进程队列
}

pub struct Processor {
    current_process: Option<Arc<ProcessControlBlock>>, // 当前正在执行的进程PCB
    idle_context: ProcessContext, // 处理器空闲状态的context
}


lazy_static! {
    pub static ref PROC_MANAGER: UPSafeCell<ProcessManager> = unsafe {UPSafeCell::new(ProcessManager::new())};
}
lazy_static! {
    pub static ref PROCESSOR: UPSafeCell<Processor> = unsafe {UPSafeCell::new(Processor::new())};
}

lazy_static! {
    pub static ref INIT_PROC: Arc<ProcessControlBlock> = Arc::new(ProcessControlBlock::new(get_app_data_by_name("init_proc").unwrap()));
}

pub fn add_initproc() {
    add_process(INIT_PROC.clone());
}

pub fn suspend_current_and_run_next() {
    // 获取当前的进程
    let current = take_current_process().unwrap();
    let mut inner = current.exclusive_borrow_inner();
    inner.status = pcb::ProcessStatus::Ready;
    let cur_ctx_ptr = &mut inner.proc_context as *mut ProcessContext;
    drop(inner);
    // 重新添加到ready队列中
    add_process(current);
    // 处理器调度下一个进程
    schedule(cur_ctx_ptr);
}

// 向进程管理器提交一个新的进程
pub fn add_process(pcb: Arc<ProcessControlBlock>) {
    PROC_MANAGER.exclusive_borrow().push(pcb);
}

// 获取进程管理器队列中的队首进程
pub fn fetch_process() -> Option<Arc<ProcessControlBlock>> {
    return PROC_MANAGER.exclusive_borrow().pop();
}

// 获取当前正在执行进程的PCB的引用
pub fn take_current_process() -> Option<Arc<ProcessControlBlock>> {
    return PROCESSOR.exclusive_borrow().take_current();
}

// 获取当前正在执行进程的PCB的拷贝
pub fn current_process() -> Option<Arc<ProcessControlBlock>> {
    return PROCESSOR.exclusive_borrow().current();
}

// 当前进程的用户空间页表satp
pub fn current_proc_satp() -> usize {
    return current_process()
    .unwrap()
    .exclusive_borrow_inner()
    .user_space_satp();
}

// 当前进程的陷入上下文
pub fn current_proc_trap_context() -> &'static mut TrapContext {
    return current_process()
    .unwrap()
    .exclusive_borrow_inner()
    .get_trap_context();
}

pub fn run_processes() {
    loop {
        let mut processor = PROCESSOR.exclusive_borrow();
        if let Some(proc) = fetch_process() {
            let mut pcb = proc.exclusive_borrow_inner();
            let idle_ctx_ptr = processor.idle_context_ptr();
            let next_ctx_ptr = &pcb.proc_context as *const ProcessContext;
            pcb.status = pcb::ProcessStatus::Running;
            drop(pcb);
            processor.current_process = Some(proc);
            drop(processor);
            unsafe {
                __switch(idle_ctx_ptr, next_ctx_ptr)
            }
        }
    }
}

// 时间片中断 或 进程主动yield触发，将Processor切换回idle
pub fn schedule(switched_ctx_ptr: *mut ProcessContext) {
    let processor = PROCESSOR.exclusive_borrow();
    let idle_ctx_ptr = processor.idle_context_ptr();
    drop(processor);
    unsafe {
        __switch(switched_ctx_ptr, idle_ctx_ptr)
    }
}


impl ProcessManager {
    pub fn new() -> Self {
        return Self{queue: VecDeque::new()};
    }
    pub fn push(&mut self, pcb: Arc<ProcessControlBlock>) {
        self.queue.push_back(pcb);
    }
    pub fn pop(&mut self) -> Option<Arc<ProcessControlBlock>> {
        return self.queue.pop_front();
    }
}

impl Processor {
    pub fn new() -> Self {
        return Self {current_process: None, idle_context: ProcessContext::new_empty_ctx()};
    }
    pub fn take_current(&mut self) -> Option<Arc<ProcessControlBlock>> {
        return self.current_process.take();
    }
    pub fn current(&self) -> Option<Arc<ProcessControlBlock>> {
        return self.current_process.as_ref().map(|proc| {Arc::clone(proc)});
    }
    fn idle_context_ptr(&self) -> *mut ProcessContext {
        let ctx = &mut self.idle_context;
        return ctx as *mut _;
    }
}








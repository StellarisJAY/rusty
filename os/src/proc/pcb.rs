use super::pid::PIDHandle;
use super::stack::KernelStack;
use crate::sync::UPSafeCell;
use crate::mem::memory_set::*;
use alloc::vec::Vec;
use alloc::sync::{Weak, Arc};
use crate::mem::address::PhysPageNumber;
use super::context::ProcessContext;
use crate::trap::context::TrapContext;
use core::cell::RefMut;
use crate::config::*;
use crate::mem::address::*;

use super::pid::alloc_pid;
use super::stack::{kernel_stack_position};
use crate::trap::trap_handler;


#[derive(Clone,Copy)]
pub enum ProcessStatus {
    New,
    Ready,
    Running,
    Exited,
    Zombie,
}

// PCB 进程控制块
#[repr(C)]
pub struct ProcessControlBlock {
    pub pid: PIDHandle, // 进程pid
    pub kernel_stack: KernelStack, // 进程内核栈
    inner: UPSafeCell<InnerPCB>, // 保证内部数据结构的唯一引用
}

#[repr(C)]
pub struct InnerPCB {
    pub trap_ctx_ppn: PhysPageNumber, // 陷入上下文的物理页号
    pub base_size: usize,
    pub proc_context: ProcessContext, // 进程上下文
    pub memory_set: MemorySet,        // 进程内存段集合
    pub parent: Option<Weak<ProcessControlBlock>>, // 父进程PCB引用
    pub children: Vec<Arc<ProcessControlBlock>>, // 子进程PCB引用集合
    pub exit_code: i32, // 进程退出代码
    pub status: ProcessStatus,
}

impl InnerPCB {
    pub fn get_trap_context(&self) -> &'static mut TrapContext {
        unsafe {
            let ptr = self.trap_ctx_ppn.get_base_address() as *mut TrapContext;
            return ptr.as_mut().unwrap();
        }
    }
    pub fn user_space_satp(&self) -> usize {
        return self.memory_set.page_table.satp_value();
    }
    pub fn get_status(&self) -> ProcessStatus {
        return self.status;
    }
}

impl ProcessControlBlock {
    pub fn exclusive_borrow_inner(&self) -> RefMut<'_, InnerPCB> {
        return self.inner.exclusive_borrow();
    }

    pub fn get_pid(&self) -> usize {
        return self.pid.0;
    }

    pub fn new(elf_data: &[u8]) -> Self {
        let (memory_set, user_stack_sp, entry_point) = MemorySet::from_elf_data(elf_data);
        // 将该任务固定的TRAP_CONTEXT虚拟地址转换为确定的物理页号
        let trap_ctx_ppn = memory_set.page_table.translate(VirtAddr::new(TRAP_CONTEXT).floor()).unwrap().page_number();
        let pid_handle = alloc_pid().unwrap();
        let kernel_stack = KernelStack::new(&pid_handle);
        let (_kernel_stack_bottom, kernel_stack_top) = kernel_stack_position(pid_handle.0);
        let inner = InnerPCB {
            trap_ctx_ppn: trap_ctx_ppn,
            base_size:elf_data.len(),
            proc_context: ProcessContext::trap_return_context(kernel_stack_top),
            memory_set: memory_set,
            parent: None,
            children: Vec::new(),
            exit_code: 0,
            status: ProcessStatus::Ready,
        };
        let trap_ctx = inner.get_trap_context();
        // 创建trap context，sepc指向app_entry
        *trap_ctx = TrapContext::task_init_context(entry_point,
        user_stack_sp,
        kernel_stack_top,
        inner.user_space_satp(),
        trap_handler as usize);
        return Self {
            kernel_stack: kernel_stack,
            pid: pid_handle,
            inner: unsafe{UPSafeCell::new(inner)},
        };
    }
}
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
use crate::mem::kernel::KERNEL_SPACE;

#[derive(Clone,Copy,PartialEq, Eq)]
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
        KERNEL_SPACE.exclusive_borrow().page_table.satp_value(),
        trap_handler as usize);
        return Self {
            kernel_stack: kernel_stack,
            pid: pid_handle,
            inner: unsafe{UPSafeCell::new(inner)},
        };
    }
    // 从当前进程PCB fork子进程PCB
    pub fn fork(self: &Arc<ProcessControlBlock>) -> Arc<ProcessControlBlock> {
        let mut inner = self.exclusive_borrow_inner();
        // 从父进程拷贝地址空间，虚拟地址范围与父进程相同
        let memory_set = MemorySet::from_existing(&inner.memory_set);
        let trap_ctx_ppn = memory_set.page_table.translate(VirtAddr::new(TRAP_CONTEXT).floor()).unwrap().page_number();
        // 分配pid和内核栈
        let pid_handle = alloc_pid().unwrap();
        let kernel_stack = KernelStack::new(&pid_handle);
        let stack_top = kernel_stack.stack_top();
        let inner_pcb = InnerPCB {
            trap_ctx_ppn: trap_ctx_ppn,
            base_size:inner.base_size,
            proc_context: ProcessContext::trap_return_context(stack_top),
            memory_set: memory_set,
            parent: Some(Arc::downgrade(self)), // 设置父进程
            children: Vec::new(),
            exit_code: 0,
            status: ProcessStatus::Ready,
        };
        let pcb = Arc::new(ProcessControlBlock{
            inner: unsafe{UPSafeCell::new(inner_pcb)},
            pid: pid_handle,
            kernel_stack: kernel_stack,
        });
        // 父进程记录子进程
        inner.children.push(pcb.clone());
        // 修改子进程的trap_ctx中的内核栈为自己的栈
        // 因为子进程的数据是从父进程地址空间拷贝的，所以这里需要切换内核栈指针
        pcb.exclusive_borrow_inner().get_trap_context().kernel_sp = stack_top;
        return pcb;
    }

    pub fn exec(&self, elf_data: &[u8]) {
        // 加载elf数据到当前进程
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf_data(elf_data);
        let trap_ctx_ppn = memory_set
            .translate(VirtAddr::new(TRAP_CONTEXT).floor())
            .unwrap()
            .page_number();

        let mut inner = self.exclusive_borrow_inner();
        inner.memory_set = memory_set;
        inner.trap_ctx_ppn = trap_ctx_ppn;
        let trap_ctx = inner.get_trap_context();
        // 修改当前进程的trap ctx，使调度可以跳转到exec任务的代码
        *trap_ctx = TrapContext::task_init_context(entry_point,
            user_sp,
            KERNEL_SPACE.exclusive_borrow().page_table.satp_value(),
            self.kernel_stack.stack_top(),
            trap_handler as usize,
        );
    }
}
use super::pid::PIDHandle;
use crate::mem::memory_set::{MemoryArea, MapType::Framed, MapPermission};
use crate::mem::address::*;
use crate::mem::kernel::KERNEL_SPACE;
use crate::config::{KERNEL_STACK_SIZE, TRAMPOLINE, PAGE_SIZE};

pub struct KernelStack {
    pid: usize,
}

impl KernelStack {
    pub fn new(pid_handle: &PIDHandle) -> Self {
        let pid = pid_handle.0;
        let mut kernel_space = KERNEL_SPACE.exclusive_borrow();
        let (stack_bottom, stack_top) = kernel_stack_position(pid);
        // 将pid对应的内核栈映射到内核空间
        kernel_space.push(MemoryArea::new(
                VirtAddr::new(stack_bottom),
                VirtAddr::new(stack_top),
                Framed, MapPermission::R | MapPermission::W),
        None);
        drop(kernel_space);
        return Self{pid};
    }

    pub fn stack_top(&self) -> usize {
        let (_, stack_top) = kernel_stack_position(self.pid);
        return stack_top;
    }

    // 栈顶压入一个数据，必须是Sized trait
    pub fn push_to_top<T>(&mut self, value: T) -> *mut T where
    T: Sized {
        let stack_top = self.stack_top();
        let ptr = (stack_top - core::mem::size_of::<T>()) as *mut T;
        unsafe {*ptr = value;}
        return ptr;
    }
}

pub fn kernel_stack_position(pid: usize) -> (usize, usize) {
    // 内核栈 = 守护页 + 栈，栈反向增长
    let stack_top = TRAMPOLINE - pid * (KERNEL_STACK_SIZE + PAGE_SIZE);
    let stack_bottom = stack_top - KERNEL_STACK_SIZE;
    return (stack_bottom, stack_top);
}

// 进程内核栈自动回收
impl Drop for KernelStack {
    fn drop(&mut self) {
        // 解除内核栈虚拟地址映射
        let (stack_bottom, _) = kernel_stack_position(self.pid);
        let mut kernel_space = KERNEL_SPACE.exclusive_borrow();
        kernel_space.remove_memory_area(VirtAddr::new(stack_bottom).floor());
        drop(kernel_space);
    }
}
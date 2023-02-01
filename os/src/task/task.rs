use crate::mem::kernel::KERNEL_SPACE;
use crate::trap::trap_handler;
use crate::mem::address::{PhysPageNumber, VirtAddr};
use crate::mem::memory_set::{MemorySet, MemoryArea, MapPermission, MapType};
use crate::config::{kernel_stack_position, TRAP_CONTEXT};
use super::context::TaskContext;
use crate::trap::context::TrapContext;

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
pub struct TaskControlBlock {
    pub status: TaskStatus,
    pub ctx: TaskContext,
    pub memory_set: MemorySet, // 任务的内存集合，其中包括了页表、分段等
    pub trap_ctx_ppn: PhysPageNumber, // 陷入上下文的物理页号
    pub base_size: usize,
}

impl TaskControlBlock {
    // 从elf数据创建任务的tcb
    // 首先将elf数据映射到内存段（用户栈、代码、跳板、guard page）
    // 通过page_table映射得到trap上下文的物理地址
    // 在内核空间映射任务的内核栈
    // 创建新的trap_context并绑定到之前获得的ppn
    pub fn new(elf_data: &[u8], app_id: usize) -> Self {
        let (memory_set, user_stack_sp, entry_point) = MemorySet::from_elf_data(elf_data);
        // 将该任务固定的TRAP_CONTEXT虚拟地址转换为确定的物理页号
        let trap_ctx_ppn = memory_set.page_table.translate(VirtAddr::new(TRAP_CONTEXT).floor()).unwrap().page_number();
        let (kernel_stack_bottom, kernel_stack_top) = kernel_stack_position(app_id);
        // 内核空间中创建该任务的内核栈区域
        let mut kernel_space = KERNEL_SPACE.exclusive_borrow();
        kernel_space.push(MemoryArea::new(VirtAddr::new(kernel_stack_bottom),
            VirtAddr::new(kernel_stack_top),
            MapType::Framed,
            MapPermission::R | MapPermission::W),
        None);
        let tcb = Self {
            status: TaskStatus::New,
            memory_set: memory_set,
            trap_ctx_ppn: trap_ctx_ppn,
            base_size: user_stack_sp,
            ctx: TaskContext::trap_return_context(kernel_stack_bottom),
        };
        let trap_ctx = tcb.get_trap_context();
        // 创建新的context
        *trap_ctx = TrapContext::task_init_context(entry_point,
        user_stack_sp,
        kernel_stack_bottom,
        kernel_space.page_table.satp_value(),
        trap_handler as usize);
        drop(kernel_space);
        debug!("application_{} loaded, page table ppn: {}", app_id, tcb.memory_set.page_table.root_ppn.0);
        return tcb;
    }
    pub fn get_trap_context(&self) -> &'static mut TrapContext {
        let ptr = self.trap_ctx_ppn.get_base_address() as *mut TrapContext;
        unsafe {
            let ctx = ptr.as_mut().unwrap();
            return ctx;
        }
    }
}

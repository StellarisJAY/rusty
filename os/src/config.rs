
// 用户栈和内核栈分别为8KiB
pub const USER_STACK_SIZE: usize = 8 * 1024;
pub const KERNEL_STACK_SIZE: usize = 8 * 1024;

// 时钟周期
pub const TIME_FREQUENCY: usize = 12500000;

pub const KERNEL_HEAP_SIZE: usize = 0x30_0000;
// RISCV物理页大小位数：4KiB 12位
pub const PAGE_SIZE_BITS: usize = 12;
pub const PAGE_SIZE: usize = 4096;
pub const PAGE_OFFSET_MASK: usize = 0xfff;

pub const MEMORY_END: usize = 0x90000000;

pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;
pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE;

// app在内核空间栈的虚拟地址
pub fn kernel_stack_position(app_id: usize) -> (usize, usize) {
    // 每个app的内核栈大小 = KERNEL_STACK_SIZE + guard page size
    let top = TRAMPOLINE - app_id * (KERNEL_STACK_SIZE + PAGE_SIZE);
    let bottom = top - KERNEL_STACK_SIZE;
    (bottom, top)
}
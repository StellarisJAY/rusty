// APP加载的基地址
pub const APP_BASE_ADDR: usize = 0x80400000;
// APP代码最大的内存空间为8KiB
pub const APP_SIZE_LIMIT: usize = 0x20000;
// APP最大数量
pub const MAX_APP_COUNT: usize = 1024;

// 用户栈和内核栈分别为8KiB
pub const USER_STACK_SIZE: usize = 8 * 1024;
pub const KERNEL_STACK_SIZE: usize = 8 * 1024;

pub const MAX_TASK_COUNT: usize = 10;

// 时钟周期
pub const TIME_FREQUENCY: usize = 12500000;

pub const KERNEL_HEAP_SIZE: usize = 1024 * 1024 * 10;
// RISCV物理页大小位数：4KiB 12位
pub const PAGE_SIZE_BITS: usize = 12;
pub const PAGE_SIZE: usize = 4096;
pub const PAGE_OFFSET_MASK: usize = 0xfff;

pub const MEMORY_END: usize = 0x82000000;
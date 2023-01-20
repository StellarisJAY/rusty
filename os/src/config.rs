// APP加载的基地址
pub const APP_BASE_ADDR: usize = 0x80400000;
// APP代码最大的内存空间为8KiB
pub const APP_SIZE_LIMIT: usize = 0x20000;
// APP最大数量
pub const MAX_APP_COUNT: usize = 1024;

// 用户栈和内核栈分别为8KiB
pub const USER_STACK_SIZE: usize = 8 * 1024;
pub const KERNEL_STACK_SIZE: usize = 8 * 1024;
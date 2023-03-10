pub mod heap_allocator;
pub mod address;
pub mod page_table;
pub mod frame_allocator;
pub mod kernel;
pub mod memory_set;
pub mod app;

pub fn init() {
    heap_allocator::init_heap();
    frame_allocator::init_frame_allocator();
    let kernel_space = kernel::KERNEL_SPACE.exclusive_borrow();
    kernel_space.reset_satp();
    drop(kernel_space);
}
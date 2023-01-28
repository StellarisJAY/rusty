use super::memory_set::{MemorySet, MemoryArea, MapType, MapPermission};
use super::address::{VirtAddr};
use crate::config::MEMORY_END;
use lazy_static::lazy_static;
use crate::sync::UPSafeCell;
use alloc::sync::Arc;

extern "C" {
    fn stext();
    fn etext();
    fn srodata();
    fn erodata();
    fn sdata();
    fn edata();
    fn sbss();
    fn ebss();
    fn boot_stack_lower_bound();
    fn boot_stack_top();
    fn ekernel();
}

lazy_static! {
    pub static ref KERNEL_SPACE: Arc<UPSafeCell<MemorySet>> = unsafe {Arc::new(UPSafeCell::new(MemorySet::new_kernel_space()))};
}

#[allow(unused)]
// 打印.text .bss .data .rodata段的地址
pub fn display_kernel_memory_layout() {
    debug!(".text section: [{:#x}, {:#x})", stext as usize, etext as usize);
    debug!(".rodata section: [{:#x}, {:#x})", srodata as usize, erodata as usize);
    debug!(".data section: [{:#x}, {:#x})", sdata as usize, edata as usize);
    debug!("boot stack: [{:#x}, {:#x}), stack size: {}KiB",
        boot_stack_lower_bound as usize,
        boot_stack_top as usize,
        (boot_stack_top as usize - boot_stack_lower_bound as usize) / 1024);
    debug!(".bss section: [{:#x}, {:#x})", sbss as usize, ebss as usize);
}

impl MemorySet {
    pub fn new_kernel_space() -> Self{
        let mut memory_set = MemorySet::new_empty();
        // 把汇编代码映射到在内核地址空间的末尾
        memory_set.map_trampoline();
        display_kernel_memory_layout();
        memory_set.push(MemoryArea::new(
                VirtAddr(stext as usize),
                VirtAddr(etext as usize),
                MapType::Direct,MapPermission::R | MapPermission::X), None);
        kernel_info!(".text memory area created");
        memory_set.push(MemoryArea::new(
                VirtAddr(srodata as usize),
                VirtAddr(erodata as usize),
                MapType::Direct, MapPermission::R), None);
        kernel_info!(".rodata memory area created");
        memory_set.push(MemoryArea::new(
                VirtAddr(sdata as usize),
                VirtAddr(edata as usize),
                MapType::Direct, MapPermission::R | MapPermission::W), None);
        kernel_info!(".data memory area created");
        memory_set.push(MemoryArea::new(
                VirtAddr(sbss as usize),
                VirtAddr(ebss as usize),
                MapType::Direct, MapPermission::R | MapPermission::W), None);
        kernel_info!(".bss memory area created");

        memory_set.push(MemoryArea::new(
                VirtAddr(ekernel as usize),
                VirtAddr(MEMORY_END),
                MapType::Direct, MapPermission::R | MapPermission::W ), None);
        kernel_info!("physical memory area created");
        return memory_set;
    }
}


#[allow(unused)]
pub fn remap_test() {
    println!("borrowd");
    let mut kernel_space = KERNEL_SPACE.exclusive_borrow();
    let mid_text: VirtAddr = VirtAddr((stext as usize + etext as usize) / 2);
    let mid_rodata: VirtAddr = VirtAddr((srodata as usize + erodata as usize) / 2);
    let mid_data: VirtAddr = VirtAddr((sdata as usize + edata as usize) / 2);
    println!("1");
    assert_eq!(
            kernel_space.page_table.vpn_to_ppn(mid_text.floor()).unwrap().is_writable(),
        false
    );
    println!("2");
    assert_eq!(
            kernel_space.page_table.vpn_to_ppn(mid_rodata.floor()).unwrap().is_writable(),
        false,
    );
    println!("3");
    assert_eq!(
            kernel_space.page_table.vpn_to_ppn(mid_data.floor()).unwrap().is_writable(),
        false,
    );
    println!("remap_test passed!");
    drop(kernel_space);
}
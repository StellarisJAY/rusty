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
    fn ekernel();
}

lazy_static! {
    pub static ref KERNEL_SPACE: Arc<UPSafeCell<MemorySet>> = unsafe {Arc::new(UPSafeCell::new(MemorySet::new_kernel_space()))};
}

impl MemorySet {
    pub fn new_kernel_space() -> Self{
        let mut memory_set = MemorySet::new_empty();
        kernel_info!("mapping kernel space, Map Mode: Direct");
        // 把汇编代码映射到在内核地址空间的末尾
        memory_set.map_trampoline();
        memory_set.push(MemoryArea::new(
                VirtAddr::new(stext as usize),
                VirtAddr::new(etext as usize),
                MapType::Direct,MapPermission::R | MapPermission::X), None);
        kernel_info!(".text memory mapped, phys addr: [{:#x}, {:#x})",stext as usize, etext as usize,);
        memory_set.push(MemoryArea::new(
                VirtAddr::new(srodata as usize),
                VirtAddr::new(erodata as usize),
                MapType::Direct, MapPermission::R), None);
        kernel_info!(".rodata memory mapped, mem range: [{:#x}, {:#x})", srodata as usize, erodata as usize);
        memory_set.push(MemoryArea::new(
                VirtAddr::new(sdata as usize),
                VirtAddr::new(edata as usize),
                MapType::Direct, MapPermission::R | MapPermission::W), None);
        kernel_info!(".data memory mapped, mem range: [{:#x}, {:#x})", sdata as usize, edata as usize);
        memory_set.push(MemoryArea::new(
                VirtAddr::new(sbss as usize),
                VirtAddr::new(ebss as usize),
                MapType::Direct, MapPermission::R | MapPermission::W), None);
        kernel_info!(".bss memory mapped, mem range: [{:#x}, {:#x})", sbss as usize, ebss as usize);

        memory_set.push(MemoryArea::new(
                VirtAddr::new(ekernel as usize),
                VirtAddr::new(MEMORY_END),
                MapType::Direct, MapPermission::R | MapPermission::W ), None);
        kernel_info!("physical memory mapped, mem range: [{:#x},{:#x})", ekernel as usize, MEMORY_END);
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
            kernel_space.page_table.translate(mid_text.floor()).unwrap().is_writable(),
        false
    );
    println!("2");
    assert_eq!(
            kernel_space.page_table.translate(mid_rodata.floor()).unwrap().is_writable(),
        false,
    );
    println!("3");
    assert_eq!(
            kernel_space.page_table.translate(mid_data.floor()).unwrap().is_writable(),
        false,
    );
    println!("remap_test passed!");
    drop(kernel_space);
}
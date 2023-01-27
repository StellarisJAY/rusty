use super::memory_set::{MemorySet, MapArea, MapType, MapPermission};
use super::address::{VirtAddr};
use crate::config::MEMORY_END;

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
        display_kernel_memory_layout();
        memory_set.push(MapArea::new(
                VirtAddr(stext as usize),
                VirtAddr(etext as usize),
                MapType::Direct,MapPermission::R | MapPermission::X), None);
        memory_set.push(MapArea::new(
                VirtAddr(srodata as usize),
                VirtAddr(erodata as usize),
                MapType::Direct, MapPermission::R), None);
        memory_set.push(MapArea::new(
                VirtAddr(sdata as usize),
                VirtAddr(edata as usize),
                MapType::Direct, MapPermission::R | MapPermission::W), None);
        memory_set.push(MapArea::new(
                VirtAddr(sbss as usize),
                VirtAddr(ebss as usize),
                MapType::Direct, MapPermission::R | MapPermission::W), None);

        memory_set.push(MapArea::new(
                VirtAddr(ekernel as usize),
                VirtAddr(MEMORY_END),
                MapType::Direct, MapPermission::R | MapPermission::W ), None);
        return memory_set;
    }
}
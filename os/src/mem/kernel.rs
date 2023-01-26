
#[allow(unused)]
// 打印.text .bss .data .rodata段的地址
pub fn display_kernel_memory_layout() {
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
    } {
        debug!(".text section: [{:#x}, {:#x})", stext as usize, etext as usize);
        debug!(".rodata section: [{:#x}, {:#x})", srodata as usize, erodata as usize);
        debug!(".data section: [{:#x}, {:#x})", sdata as usize, edata as usize);
        debug!("boot stack: [{:#x}, {:#x}), stack size: {}KiB",
        boot_stack_lower_bound as usize,
        boot_stack_top as usize,
        (boot_stack_top as usize - boot_stack_lower_bound as usize) / 1024);
        debug!(".bss section: [{:#x}, {:#x})", sbss as usize, ebss as usize);
    }
}
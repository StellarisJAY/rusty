OUTPUT_ARCH(riscv)
/* 声明入口section，此处为_start段 */
ENTRY(_start)
/* 声明内存基址 */
BASE_ADDRESS = 0x80200000;

SECTIONS
{
    /* 从内存基址开始分配 */
    . = BASE_ADDRESS;
    /* kernel内存空间开始 */
    skernel = .;
    
    /* .text段开始，.text段保存程序代码 */
    stext = .;
    /* text段包括了entry段和其他的所有text内容*/
    .text : {
        *(.text.entry)
        . = ALIGN(4K);
        strampoline = .;
        /* .text段的汇编代码在trampoline之后*/
        *(.text.trampoline);
        . = ALIGN(4K);
        *(.text .text.*)
    }
    /* 在段的末尾进行内存对齐，保证段内存是4K整数倍*/
    . = ALIGN(4K);
    /* .text段结束 */
    etext = .;
    srodata = .;
    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }

    . = ALIGN(4K);
    erodata = .;
    sdata = .;
    .data : {
        *(.data .data.*)
        *(.sdata .sdata.*)
    }

    . = ALIGN(4K);
    edata = .;
    /* bss段 bss段包括可置零的未初始化的全局变量 */
    .bss : {
        *(.bss.stack)
        sbss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
    }

    . = ALIGN(4K);
    /* bss段结束 */
    ebss = .;
    /* kernel内从空间结束 */
    ekernel = .;

    /DISCARD/ : {
        *(.eh_frame)
    }
}
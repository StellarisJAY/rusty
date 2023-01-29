    .section .text.entry //声明.text.entry段，该段作为整个系统的入口
    .global _start
_start:
    la sp, boot_stack_top //将boot后的栈顶地址赋值到sp寄存器，sp寄存器保存栈顶位置
    call rust_main //调用rust代码的rust_main函数，进入rust代码中执行
    
    .section .bss.stack //bss.stack段开始
    .global boot_stack_lower_bound
boot_stack_lower_bound: // 栈的开始地址
    .space 4096 * 16 //在bss stack段创建4096 * 16字节的空间，作为bss栈的空间
    .global boot_stack_top // global 将标签全局化
boot_stack_top: // 标记boot_stack_top，栈空间的结束地址
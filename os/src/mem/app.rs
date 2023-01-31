use super::memory_set::*;
use xmas_elf::ElfFile;
use super::address::*;
use crate::config::{PAGE_SIZE, USER_STACK_SIZE, TRAP_CONTEXT, TRAMPOLINE};

#[allow(unused)]
impl MemorySet {
    pub fn from_elf_data(data: &[u8]) -> (Self, usize, usize){
        let mut memory_set = MemorySet::new_empty();
        // 在app的虚拟地址空间结尾映射内核的汇编代码
        memory_set.map_trampoline();
        let elf = xmas_elf::ElfFile::new(data).unwrap();
        let elf_header = elf.header;
        // 检查elf开头的magic num
        let magic = elf_header.pt1.magic;
        assert_eq!(magic, [0x7f, 0x45, 0x4c, 0x46], "invalid elf!");
        // 将每一个programe header 映射到单独的内存段
        let ph_count = elf_header.pt2.ph_count();
        let mut max_end_vpn = VirtPageNumber(0);
        for i in 0..ph_count {
            let ph = elf.program_header(i).unwrap();
            if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
                // header的起始和结束虚拟地址
                let start_va: VirtAddr = VirtAddr::new(ph.virtual_addr() as usize);
                let end_va: VirtAddr = VirtAddr::new((ph.virtual_addr() + ph.mem_size()) as usize);
                let mut map_perm = MapPermission::U;
                let ph_flags = ph.flags();
                if ph_flags.is_read() { map_perm |= MapPermission::R; }
                if ph_flags.is_write() { map_perm |= MapPermission::W; }
                if ph_flags.is_execute() { map_perm |= MapPermission::X; }

                let mem_area = MemoryArea::new(
                    start_va,
                    end_va,
                    MapType::Framed,
                    map_perm,
                );
                max_end_vpn = mem_area.vpns.get_end();
                // memory_set中保存该内存段
                memory_set.push(
                    mem_area,
                    Some(&elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize])
                );
            }
        }
        // 映射用户栈
        let max_end_va = VirtAddr::from_vpn(max_end_vpn);
        let mut user_stack_bottom: usize = max_end_va.0;
        // 创建一个guard page，将用户栈与其他段隔开
        user_stack_bottom += PAGE_SIZE;
        let user_stack_top = user_stack_bottom + USER_STACK_SIZE;
        // 映射用户栈
        memory_set.push(MemoryArea::new(VirtAddr(user_stack_bottom),
            VirtAddr(user_stack_top),
            MapType::Framed,
            MapPermission::R | MapPermission::W | MapPermission::U,
        ), None);
        // 将TrapContext映射到内存set的次高地址，即跳板与栈之间的位置
        memory_set.push(MemoryArea::new(VirtAddr(TRAP_CONTEXT),
            VirtAddr(TRAMPOLINE),
            MapType::Framed,
            MapPermission::R | MapPermission::W,
        ), None);
        return (memory_set, user_stack_top, elf.header.pt2.entry_point() as usize);
    }
}
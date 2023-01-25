use bitflags::*;
use super::address::*;

bitflags! {
    pub struct PTEFlags: u8 {
        const V = 1 << 0;
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        const G = 1 << 5;
        const A = 1 << 6;
        const D = 1 << 7;
    }
}
// PPN mask 44位ppn + 2位reserve + 8位flags
// 0011 1111 1111 1111 1111 1111 1111 1111 1111 1111 1111 1100 0000 0000
const ENTRY_PPN_MASK: usize = 0x3ffffffffffc00;

// PageEntry，一个页表项大小为8字节
// 页表基地址位PT_BASE_ADDR，每个虚拟页号 i 对应的页表项地址：PT_BASE_ADDR + 8*i
// 多级页表的页表项中的PPN表示下一级页表的虚拟页号
// 最后一级页表的PPN为物理页号，物理页地址 = PPN * 4KiB + BASE_ADDR
#[derive(Clone, Copy)]
#[repr(C)]
struct PageEntry {
    pub bits: usize
}

impl PageEntry {
    pub fn new(ppn: PhysPageNumber, flags: PTEFlags) -> Self {
        Self { bits:  ppn.0 << 10 | flags.bits as usize}
    }
    pub fn empty() -> Self {
        Self{bits: 0}
    }
    pub fn page_number(&self) -> PhysPageNumber {
        (self.bits & ENTRY_PPN_MASK >> 10).into()
    }
    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits(self.bits as u8).unwrap()
    }

    pub fn is_valid(&self) -> bool {
        (self.flags() & PTEFlags::V) != PTEFlags::empty()
    }
}
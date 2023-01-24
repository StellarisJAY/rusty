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
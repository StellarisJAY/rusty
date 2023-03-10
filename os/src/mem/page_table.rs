use bitflags::*;
use super::address::*;
use super::frame_allocator::{FrameTracker, alloc_frame};
use alloc::vec::Vec;
use alloc::vec;
#[allow(unused)]
use crate::config::PAGE_SIZE;
use alloc::string::String;


const RISCV_PTE_PPN_OFFSET: usize = 10;
#[allow(unused)]
const RISCV_SATP_MODE_WIDTH: usize = 4;
const RISCV_SATP_ASID_WIDTH: usize = 16;
const RISCV_SATP_MODE_OFFSET: usize = RISCV_PPN_WIDTH + RISCV_SATP_ASID_WIDTH;
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

// PageEntry，一个页表项大小为8字节
// 非叶子节点的页表项的ppn表示下一级页表的物理页号
// 叶子节点页表项的ppn表示虚拟页映射的物理页的页号
#[derive(Clone, Copy)]
#[repr(C)]
pub struct PageTableEntry {
    pub bits: usize
}

// PageTable，三级页表根节点
// 每级页表用一个物理页保存，所以根节点需要一个root_ppn
// 一个物理页4KiB，一个页表项8B，所以一级页表可以容纳512个页表项
pub struct PageTable {
    pub root_ppn: PhysPageNumber,
    frames: Vec<FrameTracker>,
}

#[allow(unused)]
impl PageTableEntry {
    pub fn new(ppn: PhysPageNumber, flags: PTEFlags) -> Self {
        Self { bits:  ((ppn.0 & RISCV_PPN_MASK) << RISCV_PTE_PPN_OFFSET) | flags.bits as usize}
    }
    pub fn empty() -> Self {
        Self{bits: 0}
    }
    // 从pte获取ppn，pte中的ppn共44位，从低位第
    pub fn page_number(&self) -> PhysPageNumber {
        return PhysPageNumber((self.bits >> RISCV_PTE_PPN_OFFSET) & RISCV_PPN_MASK);
    }
    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits(self.bits as u8).unwrap()
    }

    fn set_ppn(&mut self, ppn: PhysPageNumber) {
        self.bits = self.bits | ((ppn.0 & RISCV_PPN_MASK) << RISCV_PTE_PPN_OFFSET);
    }

    fn set_flags(&mut self, flags: PTEFlags) {
        self.bits = self.bits | (flags.bits as usize);
    }

    pub fn is_valid(&self) -> bool {
        (self.flags() & PTEFlags::V) != PTEFlags::empty()
    }
    pub fn is_writable(&self) -> bool {
        (self.flags() & PTEFlags::W) != PTEFlags::empty()
    }
    pub fn is_readable(&self) -> bool {
        (self.flags() & PTEFlags::R) != PTEFlags::empty()
    }
    pub fn is_executable(&self) -> bool {
        (self.flags() & PTEFlags::X) != PTEFlags::empty()
    }
}

impl PageTable {
    pub fn new() -> Self {
        let frame = alloc_frame().unwrap();
        Self { root_ppn: frame.ppn, frames: vec![frame] }
    }
    pub fn map(&mut self, vpn: VirtPageNumber, ppn: PhysPageNumber, flags: PTEFlags) {
        let pte = self.find_or_create_pte(vpn).unwrap();
        assert!(!pte.is_valid(), "vpn: {} already mapped before mapping, pte ppn: {}, pt ppn: {}", vpn.0, pte.page_number().0, self.root_ppn.0);
        *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
    }
    pub fn unmap(&mut self, vpn: VirtPageNumber) {
        let pte = self.find_pte(vpn).unwrap();
        assert!(pte.is_valid(), "unmapped invalid vpn: {}", vpn.0);
        *pte = PageTableEntry::empty();
    }

    fn find_or_create_pte(&mut self, vpn: VirtPageNumber) -> Option<&mut PageTableEntry> {
        let mut ppn = self.root_ppn;
        let vpn_idxs = vpn.level_indexes();
        for i in 0..3 {
            // 获取当前一级页表的的页表项数组，然后获取vpn对应的页表项
            let pte = &mut ppn.as_pte_array()[vpn_idxs[i]];
            if i == 2 {
                return Some(pte);
            }
            // 页表项无效，创建新的页表项，并绑定到物理页
            if !pte.is_valid() {
                let frame = alloc_frame().unwrap();
                pte.set_ppn(frame.ppn);
                pte.set_flags(PTEFlags::V);
                // 所有权转移给frames，避免frameTracker被回收，导致frame重复分配
                self.frames.push(frame);
            }
            ppn = pte.page_number();
        }
        return None;
    }

    fn find_pte(&self, vpn: VirtPageNumber) -> Option<&mut PageTableEntry> {
        let mut ppn = self.root_ppn;
        let vpn_idxs = vpn.level_indexes();
        for i in 0..3 {
            let pte = &mut ppn.as_pte_array()[vpn_idxs[i]];
            if i == 2 {
                return Some(pte);
            }
            if !pte.is_valid() {
                return None;
            }
            ppn = pte.page_number();
        }
        return None;
    }

    #[allow(unused)]
    // 从satp寄存器构建页表
    pub fn from_satp_register(satp: usize) -> Self {
        Self {
            root_ppn: PhysPageNumber(satp & ((1usize << 44) - 1)),
            frames: Vec::new(),
        }
    }
    pub fn translate(&self, vpn: VirtPageNumber) -> Option<PageTableEntry> {
        self.find_pte(vpn)
            .map(|pte| {*pte})
    }

    // 将当前页表转换成satp寄存器值
    // satp寄存器：8位 + 44位页表所在的ppn
    pub fn satp_value(&self) -> usize {
        return (8 << RISCV_SATP_MODE_OFFSET) | (self.root_ppn.0);
    }
    // 虚拟地址到物理地址的转换
    pub fn translate_virt_addr(&self, va: VirtAddr) -> Option<usize> {
        let page_offset = va.page_offset();
        let vpn = va.floor();
        return self.translate(vpn)
        .map(|pte| {
            pte.page_number().as_phys_addr(page_offset).0
        });
    }
}

pub fn translated_byte_buffer(
        satp: usize,
        ptr: *const u8,
        len: usize
) -> Vec<&'static mut[u8]> {
    let page_table = PageTable::from_satp_register(satp);
    let mut start = ptr as usize;
    let end = start + len;
    let mut v: Vec<&'static mut[u8]> = Vec::new();
    while start < end {
        let start_va = VirtAddr::new(start);
        let mut vpn = start_va.floor();
        let ppn = page_table
            .translate(vpn)
            .unwrap()
            .page_number();
        vpn.step();
        let mut end_va = VirtAddr::from_vpn(vpn);
        end_va = end_va.min(VirtAddr::new(end));
        v.push(&mut ppn.as_bytes_array()[start_va.page_offset()..end_va.page_offset()]);
        start = end_va.into();
    }
    return v;
}

// 从指定地址空间转换指针为String
pub fn translate_string(satp: usize, ptr: *const u8) -> String {
    let page_table = PageTable::from_satp_register(satp);
    let mut str = String::new();
    let mut va: usize = ptr as usize;
    loop {
        let phys_addr = page_table.translate_virt_addr(VirtAddr::new(va)).unwrap();
        unsafe {
            let ch: u8 = *(phys_addr as *const u8);
            if ch != 0 {
                str.push(ch as char);
                va += 1;
            }else {
                break;
            }
        }
    }
    return str;
}

// 将指针的虚拟地址转换到对应地址空间的物理地址
pub fn translate_ptr<T>(satp: usize, ptr: *mut T) -> &'static mut T
where T: Sized{
    unsafe {
        let page_table = PageTable::from_satp_register(satp);
        let addr = page_table.translate_virt_addr(VirtAddr::new(ptr as usize)).unwrap();
        let ptr = addr as *mut T;
        return ptr.as_mut().unwrap();
    }
}
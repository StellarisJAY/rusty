use bitflags::*;
use super::address::*;
use super::frame_allocator::{FrameTracker, alloc_frame};
use alloc::vec::Vec;
use alloc::vec;

const RISCV_PTE_PPN_OFFSET: usize = 10;

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

impl PageTableEntry {
    pub fn new(ppn: PhysPageNumber, flags: PTEFlags) -> Self {
        Self { bits:  (ppn.0 & RISCV_PPN_MASK) << RISCV_PTE_PPN_OFFSET | flags.bits as usize}
    }
    pub fn empty() -> Self {
        Self{bits: 0}
    }
    // 从pte获取ppn，pte中的ppn共44位，从低位第
    pub fn page_number(&self) -> PhysPageNumber {
        return PhysPageNumber(self.bits >> RISCV_PTE_PPN_OFFSET & RISCV_PPN_MASK);
    }
    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits(self.bits as u8).unwrap()
    }

    pub fn is_valid(&self) -> bool {
        (self.flags() & PTEFlags::V) != PTEFlags::empty()
    }
    pub fn is_writable(&self) -> bool {
        (self.flags() & PTEFlags::W) != PTEFlags::empty()
    }
}

impl PageTable {
    pub fn new() -> Self {
        let frame = alloc_frame().unwrap();
        Self { root_ppn: frame.ppn, frames: vec![frame] }
    }
    pub fn map(&mut self, vpn: VirtPageNumber, ppn: PhysPageNumber, flags: PTEFlags) {
        debug!("page table mapping vpn: {}, ppn: {}", vpn.0, ppn.0);
        let pte = self.find_or_create_pte(vpn).unwrap();
        //assert!(!pte.is_valid(), "vpn: {} already mapped before mapping, pte ppn: {}", vpn.0, pte.page_number().0);
        *pte = PageTableEntry::new(ppn, PTEFlags::V);
        debug!("page table entry mapped, vpn: {}, ppn: {}", vpn.0, pte.page_number().0);
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
                debug!("found pte, pte ppn: {}, bits: {:#b}", pte.page_number().0, pte.bits);
                return Some(pte);
            }
            // 页表项无效，创建新的页表项，并绑定到物理页
            if !pte.is_valid() {
                let frame = alloc_frame().unwrap();
                *pte = PageTableEntry::new(frame.ppn, PTEFlags::V);
                self.frames.push(FrameTracker{ppn: frame.ppn});
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
    pub fn vpn_to_ppn(&self, vpn: VirtPageNumber) -> Option<PageTableEntry> {
        self.find_pte(vpn)
            .map(|pte| {pte.clone()})
    }

    pub fn satp_value(&self) -> usize {
        8 << 60 | (self.root_ppn.0 & 0xfffffffffff)
    }
}
use super::address::{VirtPageNumber, VirtAddr, PhysPageNumber, PhysAddr, VPNRange};
use crate::config::PAGE_SIZE;
use super::frame_allocator::{FrameTracker, alloc_frame};
use alloc::collections::BTreeMap;
use bitflags::bitflags;
use super::page_table::{PageTable, PTEFlags, PageTableEntry};
use alloc::vec::Vec;
use riscv::register::satp;
use core::arch::asm;
use crate::config::TRAMPOLINE;

#[derive(Clone)]
pub enum MapType {
    Direct,
    Framed,
}

// MemoryArea 一个内存段
// VPNRange定义了段内存的虚拟页号范围
pub struct MemoryArea {
    pub vpns: VPNRange,
    mapped_frames: BTreeMap<VirtPageNumber, FrameTracker>,
    map_type: MapType,
    map_perm: MapPermission,
}


bitflags! {
    pub struct MapPermission: u8 {
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
    }
}

pub struct MemorySet {
    pub page_table: PageTable,
    areas: Vec<MemoryArea>,
}


impl MemoryArea {
    pub fn new(start_va: VirtAddr, end_va: VirtAddr, map_type: MapType, perm: MapPermission) -> Self {
        let start_vpn = start_va.floor();
        let end_vpn = end_va.ceil();
        return Self { vpns: VPNRange::new(start_vpn, end_vpn), mapped_frames: BTreeMap::new(), map_type: map_type, map_perm: perm };
    }

    pub fn from_existing(other: &MemoryArea) -> Self {
        return Self{vpns: VPNRange::new(other.vpns.get_start(), other.vpns.get_end()), mapped_frames: BTreeMap::new(),
            map_type: other.map_type.clone(),
            map_perm: other.map_perm};
    }

    // 将该段与页表映射
    pub fn map(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpns {
            self.map_vpn(page_table, vpn);
        }
    }
    // 解除该段与页表的映射
    pub fn unmap(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpns {
            self.unmap_vpn(page_table, vpn);
        }
    }
    // 将数据拷贝到段内存中
    // 从vpn 0开始，将数据分成4KiB的若干个页，通过页表获取vpn对应的物理页，并将数据拷贝到物理页中
    pub fn copy_data(&mut self, page_table: &mut PageTable, data: &[u8]) {
        let mut start: usize = 0;
        let mut current_vpn = self.vpns.get_start();
        let len = data.len();
        loop {
            let src = &data[start..len.min(start + PAGE_SIZE)];
            let dst = &mut page_table
                .translate(current_vpn)
                .unwrap()
                .page_number()
                .as_bytes_array()[..src.len()];
            dst.copy_from_slice(src);
            start += PAGE_SIZE;
            if start >= len {
                break;
            }
            current_vpn.step();
        }
    }
    // 将一个vpn映射到该内存段中
    pub fn map_vpn(&mut self, page_table: &mut PageTable, vpn: VirtPageNumber) {
        let ppn: PhysPageNumber;
        match self.map_type {
            // 直接映射，vpn直接当作物理页号
            MapType::Direct => {
                ppn = PhysPageNumber(vpn.0);
            },
            // 分配物理页，然后映射将vpn与ppn在页表映射
            MapType::Framed => {
                let frame = alloc_frame().unwrap();
                ppn = frame.ppn;
                self.mapped_frames.insert(vpn, frame);
            }
        }
        let flags = PTEFlags::from_bits(self.map_perm.bits).unwrap();
        page_table.map(vpn, ppn, flags)
    }
    // 将一个vpn与当前段解除映射
    pub fn unmap_vpn(&mut self, page_table: &mut PageTable, vpn: VirtPageNumber) {
        match self.map_type {
            MapType::Framed => {
                self.mapped_frames.remove(&vpn);
            },
            _ => {},
        }
        page_table.unmap(vpn);
    }
}

impl MemorySet {
    pub fn new_empty() -> Self {
        return Self { page_table: PageTable::new(), areas: Vec::new() };
    }
    // 从已存在的MemorySet创建新的MemorySet，用于fork进程
    pub fn from_existing(other: &Self) -> Self {
        let mut mem_set = Self::new_empty();
        mem_set.map_trampoline();
        for area in other.areas.iter() {
            let new_area = MemoryArea::from_existing(&area);
            mem_set.push(new_area, None);
            // 复制数据到新的数据集
            // TODO 实现CopyOnWrite机制
            for vpn in area.vpns {
                let src_ppn = other.translate(vpn).unwrap().page_number();
                let dst_ppn = mem_set.translate(vpn).unwrap().page_number();
                dst_ppn.as_bytes_array().copy_from_slice(src_ppn.as_bytes_array());
            }
        }
        return mem_set;
    }

    // push一个内存段到内存合集中
    pub fn push(&mut self, mut area: MemoryArea, data: Option<&[u8]>) {
        area.map(&mut self.page_table);
        if let Some(d) = data {
            area.copy_data(&mut self.page_table, d);
        }
        self.areas.push(area);
    }
    pub fn reset_satp(&self) {
        let satp = self.page_table.satp_value();
        satp::write(satp);
        // 刷新TLB
        unsafe{
            asm!("sfence.vma");
        }
    }
    // 将trampoline的汇编代码地址映射到 地址空间中的固定位置
    pub fn map_trampoline(&mut self) {
        extern "C" {
            fn strampoline();
        }
        // strampoline为汇编代码的物理地址，TRAMPOLINE是虚拟地址
        // 将vpn与ppn在当前的地址空间中绑定
        self.page_table.map(VirtAddr::new(TRAMPOLINE).floor(), PhysAddr::new(strampoline as usize).floor(), PTEFlags::R | PTEFlags::X);
    }

    pub fn translate(&self, vpn: VirtPageNumber) -> Option<PageTableEntry> {
        self.page_table.translate(vpn)
    }

    // 删除集合中的一个内存段
    pub fn remove_memory_area(&mut self, start_vpn: VirtPageNumber) {
        let mut found: bool = false;
        let mut idx: usize = 0;
        // 找到从指定vpn开始的内存段
        for (i, area) in self.areas.iter().enumerate() {
            if area.vpns.get_start().0 == start_vpn.0 {
                idx = i;
                found = true;
                break;
            }
        }
        if found {
            let index = idx as usize;
            // area的所有权从vec移除，函数结束后生命周期结束被回收
            // 进而触发area中的所有物理页被回收
            let mut area = self.areas.remove(index);
            area.unmap(&mut self.page_table);
        }else {
            panic!("unmap non-exist memory area, memory_set: {}, start vpn: {}", self.page_table.root_ppn.0, start_vpn.0);
        }
    }
    
    
    pub fn recycle_memory_set(&mut self) {
        // 释放areas的所有权，导致MemoryArea和area中的Frame自动回收
        self.areas.clear();
    }
}

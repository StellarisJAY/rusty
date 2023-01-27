use super::address::{VirtPageNumber, VirtAddr, PhysAddr, PhysPageNumber};
use crate::config::PAGE_SIZE;
use super::frame_allocator::{FrameTracker, alloc_frame, dealloc_frame};
use alloc::collections::BTreeMap;
use bitflags::bitflags;
use super::page_table::{PageTable, PTEFlags};
use alloc::vec::Vec;

pub enum MapType {
    Direct,
    Framed,
}

// MapArea 一个内存段
// VPNRange定义了段内存的虚拟页号范围
pub struct MapArea {
    vpns: VPNRange,
    mapped_frames: BTreeMap<VirtPageNumber, FrameTracker>,
    map_type: MapType,
    map_perm: MapPermission,
}

#[derive(Clone, Copy)]
struct VPNRange {
    start_vpn: VirtPageNumber,
    end_vpn: VirtPageNumber,
    current: VirtPageNumber,
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
    page_table: PageTable,
    areas: Vec<MapArea>,
}

impl VPNRange {
    fn new(start: VirtPageNumber, end: VirtPageNumber)->Self {
        return Self { start_vpn: start, end_vpn: end, current: start };
    }
}

impl MapArea {
    pub fn new(start_va: VirtAddr, end_va: VirtAddr, map_type: MapType, perm: MapPermission) -> Self {
        let start_vpn = start_va.floor();
        let end_vpn = end_va.floor();
        return Self { vpns: VPNRange::new(start_vpn, end_vpn), mapped_frames: BTreeMap::new(), map_type: map_type, map_perm: perm };
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
        let mut data_pos:usize = 0;
        let mut current_vpn = self.vpns.start_vpn;
        let data_len = data.len();
        loop {
            // 获取4KiB的数据切片
            let src = &data[data_pos..data_pos + PAGE_SIZE];
            // 通过当前的vpn获取一个物理页号，并获取物理页对应的数据切片
            let dst = page_table.vpn_to_ppn(current_vpn).unwrap().page_number().as_bytes_array();
            dst.copy_from_slice(src);
            data_pos += PAGE_SIZE;
            if data_pos >= data_len {
                break;
            }
            current_vpn.0 += 1;
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
                self.mapped_frames.insert(vpn, FrameTracker{ppn: ppn});
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
    pub fn push(&mut self, area: MapArea, data: Option<&[u8]>) {
        area.map(&mut self.page_table);
        if let Some(d) = data {
            area.copy_data(&mut self.page_table, d);
        }
        self.areas.push(area);
    }
}



impl Iterator for VPNRange {
    type Item = VirtPageNumber;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.end_vpn {
            return None;
        }
        let result = self.current;
        self.current.0 += 1;
        return Some(result);
    }
}
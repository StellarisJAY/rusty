use crate::config::{PAGE_OFFSET_MASK, PAGE_SIZE_BITS, PAGE_SIZE};
use super::page_table::PageTableEntry;

const RISCV_PPN_WIDTH: usize = 44;
// RISCV物理地址长度，56位。
const RISCV_PA_WIDTH: usize = 56;
// RISCV页表号长度27位
const RISCV_VPN_WIDTH: usize = 27;
// RISCV虚拟地址长度39位，最多表示512GiB的地址空间
const RISCV_VA_WIDTH: usize = 39;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysAddr(pub usize);
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysPageNumber(pub usize);
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtAddr(pub usize);
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtPageNumber(pub usize);

impl PhysAddr {
    // 获取物理地址中的页内偏移
    pub fn page_offset(&self) -> usize {
        self.0 & PAGE_OFFSET_MASK
    }
    // 物理地址向下取整获得物理页号
    pub fn floor(&self) -> PhysPageNumber { PhysPageNumber(self.0 / PAGE_SIZE) }
    // 物理地址向上取整获得物理页号
    pub fn ceil(&self) -> PhysPageNumber { PhysPageNumber((self.0 + PAGE_SIZE - 1) / PAGE_SIZE) }
}

impl VirtAddr {
    pub fn page_offset(&self) -> usize {
        self.0 & PAGE_OFFSET_MASK
    }
}


impl PhysPageNumber {
    // 将一个物理页作为mutable切片返回
    pub fn as_bytes_array(&self) -> &'static mut [u8] {
        let start_ptr = ((self.0 * PAGE_SIZE) as usize)  as *mut u8;
        unsafe {core::slice::from_raw_parts_mut(start_ptr, PAGE_SIZE)}
    }
    // 将一个物理页作为多级页表的页表项数组返回
    // 一个物理页（4KiB）可以容纳 512个页表项（8 字节）
    pub fn as_pte_array(&self) -> &'static mut [PageTableEntry] {
        let ptr = ((self.0 * PAGE_SIZE) as usize)  as *mut PageTableEntry;
        unsafe{core::slice::from_raw_parts_mut(ptr, PAGE_SIZE / 8)}
    }
}

impl VirtPageNumber {
    // 获取三级页表的三个虚拟页号
    // 每个虚拟页号为9位，可以映射512个物理页
    pub fn level_indexes(&self) -> [usize; 3] {
        let mut vpn = self.0;
        let mut idxs: [usize; 3] = [0; 3];
        // 低位是更高级的页表，所以需要rev
        for i in (0..3).rev() {
            idxs[i] = vpn & 0x1ff;
            vpn = vpn >> 9;
        }
        return idxs;
    }
}

impl From<usize> for PhysAddr {
    fn from(value: usize) -> Self {
        Self(value & (1<<RISCV_PA_WIDTH - 1))
    }
}
impl From<usize> for PhysPageNumber {
    fn from(value: usize) -> Self {
        Self(value & (1<<RISCV_PPN_WIDTH - 1))
    }
}
impl From<PhysAddr> for usize {
    fn from(value: PhysAddr) -> usize {
        value.0
    }
}
impl From<PhysPageNumber> for usize {
    fn from(value: PhysPageNumber) -> Self {
        value.0
    }
}

impl From<usize> for VirtAddr {
    fn from(value: usize) -> Self {
        Self(value & (1 << RISCV_VA_WIDTH - 1))
    }
}

impl From<usize> for VirtPageNumber {
    fn from(value: usize) -> Self {
        Self(value & (1<<RISCV_VPN_WIDTH - 1))
    }
}

impl From<VirtAddr> for usize {
    fn from(value: VirtAddr) -> usize {
        value.0
    }
}

impl From<VirtPageNumber> for usize {
    fn from(value: VirtPageNumber) -> usize {
        value.0
    }
}


impl From<PhysAddr> for PhysPageNumber {
    fn from(v: PhysAddr) -> Self {
        assert_eq!(v.page_offset(), 0);
        v.floor()
    }
}

impl From<PhysPageNumber> for PhysAddr {
    fn from(v: PhysPageNumber) -> Self { Self(v.0 << PAGE_SIZE_BITS) }
}
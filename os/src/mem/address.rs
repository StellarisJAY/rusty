use crate::config::{PAGE_OFFSET_MASK, PAGE_SIZE_BITS, PAGE_SIZE};

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
    pub fn page_offset(&self) -> usize {
        self.0 & PAGE_OFFSET_MASK
    }
    pub fn floor(&self) -> PhysPageNumber { PhysPageNumber(self.0 / PAGE_SIZE) }
    pub fn ceil(&self) -> PhysPageNumber { PhysPageNumber((self.0 + PAGE_SIZE - 1) / PAGE_SIZE) }
}

impl VirtAddr {
    pub fn page_offset(&self) -> usize {
        self.0 & PAGE_OFFSET_MASK
    }
    pub fn floor(&self) -> VirtPageNumber { VirtPageNumber(self.0 / PAGE_SIZE) }
    pub fn ceil(&self) -> VirtPageNumber { VirtPageNumber((self.0 + PAGE_SIZE - 1) / PAGE_SIZE) }
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
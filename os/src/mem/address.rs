use crate::config::{PAGE_OFFSET_MASK, PAGE_SIZE, PAGE_SIZE_BITS};
use super::page_table::PageTableEntry;

pub const RISCV_PPN_WIDTH: usize = 44;
// RISCV物理地址长度，56位。
pub const RISCV_PA_WIDTH: usize = 56;
// RISCV页表号长度27位
pub const RISCV_VPN_WIDTH: usize = 27;
// RISCV虚拟地址长度39位，最多表示512GiB的地址空间
pub const RISCV_VA_WIDTH: usize = 39;

pub const RISCV_PPN_MASK: usize = (1<<RISCV_PPN_WIDTH) - 1;
pub const RISCV_VPN_MASK: usize = (1<<RISCV_VPN_WIDTH) - 1;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysAddr(pub usize);
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysPageNumber(pub usize);
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtAddr(pub usize);
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtPageNumber(pub usize);


impl PhysAddr {
    pub fn new(val: usize) -> Self {
        return Self(val & ((1 << RISCV_PA_WIDTH) - 1));
    }
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
    pub fn new(val: usize) -> Self {
        return Self(val & ((1<<RISCV_VA_WIDTH) - 1));
    }
    pub fn page_offset(&self) -> usize {
        self.0 & PAGE_OFFSET_MASK
    }
    // 物理地址向下取整获得物理页号
    pub fn floor(&self) -> VirtPageNumber { VirtPageNumber(self.0 / PAGE_SIZE) }
    // 物理地址向上取整获得物理页号
    pub fn ceil(&self) -> VirtPageNumber { VirtPageNumber((self.0 + PAGE_SIZE - 1) / PAGE_SIZE) }
    // 从虚拟页号获取虚拟地址基地址，RISC-V的虚拟页号只有39位
    pub fn from_vpn(vpn: VirtPageNumber) -> Self {
        return Self(vpn.0  << PAGE_SIZE_BITS);
    }
}


impl PhysPageNumber {
    // 将一个物理页作为mutable切片返回
    pub fn as_bytes_array(&self) -> &'static mut [u8] {
        let start_ptr = self.get_base_address()  as *mut u8;
        unsafe {core::slice::from_raw_parts_mut(start_ptr, PAGE_SIZE)}
    }
    // 将一个物理页作为多级页表的页表项数组返回
    // 一个物理页（4KiB）可以容纳 512个页表项（8 字节）
    pub fn as_pte_array(&self) -> &'static mut [PageTableEntry] {
        let ptr = self.get_base_address() as *mut PageTableEntry;
        let array = unsafe{core::slice::from_raw_parts_mut(ptr, 512)};
        return array;
    }
    // 物理页基地址
    pub fn get_base_address(&self) -> usize {
        let base_addr = self.0 << PAGE_SIZE_BITS;
        return base_addr;
    }
    
    // 物理页转换物理地址
    pub fn as_phys_addr(&self, page_offset: usize) -> PhysAddr {
        assert!(page_offset >= PAGE_SIZE, "page offset overflow");
        return PhysAddr::new(self.get_base_address() & page_offset);
    }
}

impl From<VirtAddr> for usize {
    fn from(value: VirtAddr) -> Self {
        if value.0 >= 1<<(RISCV_VA_WIDTH-1) {
            return value.0 | (!((1<<(RISCV_VA_WIDTH - 1))-1));
        }else {
            return value.0;
        }
    }
}

impl VirtPageNumber {
    // 获取三级页表的三个虚拟页号
    // 每个虚拟页号为9位，可以映射512个物理页
    pub fn level_indexes(&self) -> [usize; 3] {
        let mut vpn = self.0;
        let mut idx = [0usize; 3];
        for i in (0..3).rev() {
            idx[i] = vpn & 511;
            vpn >>= 9;
        }
        idx
    }
    pub fn step(&mut self) {
        self.0 += 1;
    }
}



pub trait StepByOne {
    fn step(&mut self);
}
impl StepByOne for VirtPageNumber {
    fn step(&mut self) {
        self.0 += 1;
    }
}


#[derive(Copy, Clone)]
/// a simple range structure for type T
pub struct SimpleRange<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd,
{
    l: T,
    r: T,
}
impl<T> SimpleRange<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd,
{
    pub fn new(start: T, end: T) -> Self {
        Self { l: start, r: end }
    }
    pub fn get_start(&self) -> T {
        self.l
    }
    pub fn get_end(&self) -> T {
        self.r
    }
}
impl<T> IntoIterator for SimpleRange<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd,
{
    type Item = T;
    type IntoIter = SimpleRangeIterator<T>;
    fn into_iter(self) -> Self::IntoIter {
        SimpleRangeIterator::new(self.l, self.r)
    }
}
/// iterator for the simple range structure
pub struct SimpleRangeIterator<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd,
{
    current: T,
    end: T,
}
impl<T> SimpleRangeIterator<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd,
{
    pub fn new(l: T, r: T) -> Self {
        Self { current: l, end: r }
    }
}
impl<T> Iterator for SimpleRangeIterator<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.end {
            None
        } else {
            let t = self.current;
            self.current.step();
            Some(t)
        }
    }
}

/// a simple range structure for virtual page number
pub type VPNRange = SimpleRange<VirtPageNumber>;
use super::address::{PhysPageNumber, PhysAddr};
use alloc::vec::Vec;
use crate::sync::UPSafeCell;
use lazy_static::lazy_static;
use crate::config::MEMORY_END;
// 物理帧分配器trait
trait FrameAllocator {
    fn new() -> Self;
    // alloc分配物理帧，返回一个物理页号
    fn alloc(&mut self) -> Option<PhysPageNumber>;
    // dealloc回收一个物理页
    fn dealloc(&mut self, ppn: PhysPageNumber);
}

// 栈式物理页分配器
pub struct StackFrameAllocator {
    current: usize, // 当前的栈顶位置
    end: usize, // 栈内存结束位置
    recycled: Vec<usize>,
}

#[derive(Clone)]
pub struct FrameTracker {
    pub ppn: PhysPageNumber,
}

type FrameAllocatorImpl = StackFrameAllocator;

lazy_static! {
    pub static ref FRAME_ALLOCATOR: UPSafeCell<FrameAllocatorImpl> = unsafe {
        UPSafeCell::new(FrameAllocatorImpl::new())
    };
}

// 初始化物理页分配器
pub fn init_frame_allocator() {
    extern "C" {
        fn ekernel();
    }
    let mut allocator = FRAME_ALLOCATOR.exclusive_borrow();
    let (start_addr, end_addr) = (PhysAddr::new(ekernel as usize), PhysAddr::new(MEMORY_END));
    // 内核同样需要占用物理内存页，所以从内核结束地址ekernel计算可分配内存的初始页号
    allocator.init(start_addr.ceil(),  end_addr.ceil());
    kernel_info!("physical memory frame allocator space: [{:#x}, {:#x}), size: {}, pages: {}", start_addr.0, end_addr.0, end_addr.0 - start_addr.0, allocator.end - allocator.current);
    drop(allocator);
}

// 分配一个物理页，返回Optional
pub fn alloc_frame() -> Option<FrameTracker> {
    let mut allocator = FRAME_ALLOCATOR.exclusive_borrow();
    let ppn = allocator.alloc().unwrap();
    drop(allocator);
    return Some(FrameTracker::new(ppn));
}

// 释放一个物理页，错误的页号会导致panic
pub fn dealloc_frame(ppn: PhysPageNumber) {
    let mut allocator = FRAME_ALLOCATOR.exclusive_borrow();
    allocator.dealloc(ppn);
    drop(allocator);
}


impl FrameAllocator for StackFrameAllocator {
    fn new() -> Self {
        return Self {current: 0,end: 0,recycled: Vec::new()};
    }
    fn alloc(&mut self) -> Option<PhysPageNumber> {
        // 尝试从已回收的页中分配
        if let Some(ppn) = self.recycled.pop() {
            return Some(PhysPageNumber(ppn));
        }else {
            // 没有空闲的页
            if self.current == self.end {
                return None;
            }
            // 分配当前栈顶的页
            let ppn: PhysPageNumber = PhysPageNumber(self.current);
            self.current += 1;
            return Some(ppn);
        }
    }
    fn dealloc(&mut self, ppn: PhysPageNumber) {
        let ppn = ppn.0;
        if ppn >= self.current || self.recycled.iter().find(|&a| {return *a == ppn;}).is_some() {
            panic!("invalid page number : {}, current stack page number: {}", ppn, self.current);
        }
        self.recycled.push(ppn);
    }
}

impl StackFrameAllocator {
    // 初始化分配器的起始和末尾页号
    fn init(&mut self, low: PhysPageNumber, high: PhysPageNumber) {
        self.current = low.0;
        self.end = high.0;
    }
}

impl FrameTracker {
    fn new(ppn: PhysPageNumber) -> Self {
        // 将物理页内存清零
        let bytes = ppn.as_bytes_array();
        bytes.fill(0);
        return FrameTracker { ppn: ppn };
    }
}

// 实现FrameTracker的Drop trait
// 使应用程序可以通过drop函数释放内存
impl Drop for FrameTracker {
    fn drop(&mut self) {
        dealloc_frame(self.ppn);
    }
}

#[allow(unused)]
pub fn frame_allocator_test() {
    let mut v: Vec<FrameTracker> = Vec::new();
    for i in 0..5 {
        let frame = alloc_frame().unwrap();
        println!("ppn: {:?}", frame.ppn.0);
        v.push(frame);
    }
    v.clear();
    for i in 0..5 {
        let frame = alloc_frame().unwrap();
        println!("{:?}", frame.ppn.0);
        v.push(frame);
    }
    drop(v);
    println!("frame_allocator_test passed!");
}

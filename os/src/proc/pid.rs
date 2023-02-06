use alloc::vec::Vec;
use lazy_static::lazy_static;
use crate::sync::UPSafeCell;

pub struct PIDHandle(pub usize);

struct PIDAllocator {
    current: usize,
    end: usize,
    recycled: Vec<usize>,
}

const PID_LIMIT: usize = 1024;

impl PIDAllocator {
    fn new(start: usize, end: usize) -> Self {
        assert!(end > start, "invalid pid range");
        return PIDAllocator { current: start, end: end, recycled: Vec::new() };
    }
    fn alloc(&mut self) -> Option<PIDHandle> {
        if let Some(pid) = self.recycled.pop() {
            return Some(PIDHandle(pid));
        }else {
            if self.current == self.end {
                return None;
            }
            let pid = self.current;
            self.current += 1;
            return Some(PIDHandle(pid));
        }
    }
    fn dealloc(&mut self, pid: usize) {
        self.recycled.push(pid);
    }
}


lazy_static! {
    static ref PID_ALLOCATOR: UPSafeCell<PIDAllocator> = unsafe {
        UPSafeCell::new(PIDAllocator::new(0, PID_LIMIT))
    };
}

pub fn alloc_pid() -> Option<PIDHandle> {
    return PID_ALLOCATOR.exclusive_borrow().alloc();
}

impl Drop for PIDHandle {
    fn drop(&mut self) {
        PID_ALLOCATOR.exclusive_borrow().dealloc(self.0);
    }
}

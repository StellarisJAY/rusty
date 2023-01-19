use core::cell::{RefCell, RefMut};

// 重新包装RefCell，然后实现Sync Trait
pub struct UPSafeCell<T> {
    inner: RefCell<T>,
}

unsafe impl<T> Sync for UPSafeCell<T> {}

impl<T> UPSafeCell<T> {
    pub unsafe fn new(value: T) -> Self {
        return UPSafeCell{inner: RefCell::new(value)};
    }
    
    pub fn exclusive_borrow(&self) -> RefMut<'_, T> {
        return self.inner.borrow_mut();
    }
}
//! Common device driver code.

use core::{marker::PhantomData, ops};

pub struct MmioDerefWrapper<T> {
    base_addr: usize,
    phantom: PhantomData<fn() -> T>,
}

impl<T> MmioDerefWrapper<T> {
    /// Create an instance.
    pub const fn new(base_addr: usize) -> Self {
        Self {
            base_addr,
            phantom: PhantomData,
        }
    }
}

impl<T> ops::Deref for MmioDerefWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.base_addr as *const _) }
    }
}

//! Common device driver code.

use core::{fmt, marker::PhantomData, ops};

pub struct MmioDerefWrapper<T> {
    base_addr: usize,
    phantom: PhantomData<fn() -> T>,
}

impl<T> MmioDerefWrapper<T> {
    /// Create an instance.
    pub const unsafe fn new(base_addr: usize) -> Self {
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

/// A wrapper type for usize with integrated range bound check.
#[derive(Copy, Clone)]
pub struct BoundedUsize<const MAX_INCLUSIVE: usize>(usize);

impl<const MAX_INCLUSIVE: usize> BoundedUsize<{ MAX_INCLUSIVE }> {
    pub const MAX_INCLUSIVE: usize = MAX_INCLUSIVE;

    /// Creates a new instance if number <= MAX_INCLUSIVE.
    pub const fn new(number: usize) -> Self {
        assert!(number <= MAX_INCLUSIVE);

        Self(number)
    }

    /// Return the wrapped number.
    pub const fn get(self) -> usize {
        self.0
    }
}

impl<const MAX_INCLUSIVE: usize> fmt::Display for BoundedUsize<{ MAX_INCLUSIVE }> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

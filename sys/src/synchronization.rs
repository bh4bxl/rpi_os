//! Synchronization primitives.

use core::cell::UnsafeCell;

/// Synchronization interfaces.
pub mod interface {
    /// Any object implementing this trait guarantees exclusive access to the data wrapped within
    /// the Mutex for the duration of the provided closure.
    pub trait Mutex {
        /// The type of the data that is wrapped by this mutex.
        type Data;

        /// Locks the mutex and grants the closure temporary mutable access to the wrapped data.
        fn lock<'a, R>(&'a self, f: impl FnOnce(&'a mut Self::Data) -> R) -> R;
    }

    /// A reader-writer exclusion type.
    pub trait ReadWriteEx {
        /// The type of encapsulated data.
        type Data;

        /// Grants temporary mutable access to the encapsulated data.
        fn write<'a, R>(&'a self, f: impl FnOnce(&'a mut Self::Data) -> R) -> R;

        /// Grants temporary immutable access to the encapsulated data.
        fn read<'a, R>(&'a self, f: impl FnOnce(&'a Self::Data) -> R) -> R;
    }
}

/// A pseudo-lock for teaching purposes.
pub struct IrqSafeNullLock<T>
where
    T: ?Sized,
{
    data: UnsafeCell<T>,
}

unsafe impl<T> Send for IrqSafeNullLock<T> where T: ?Sized + Send {}
unsafe impl<T> Sync for IrqSafeNullLock<T> where T: ?Sized + Send {}

impl<T> IrqSafeNullLock<T> {
    /// Create an instance.
    pub const fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
        }
    }
}

impl<T> interface::Mutex for IrqSafeNullLock<T> {
    type Data = T;

    fn lock<'a, R>(&'a self, f: impl FnOnce(&'a mut Self::Data) -> R) -> R {
        let data = unsafe { &mut *self.data.get() };

        // Execute the closure while IRQs are masked.
        f(data)
    }
}

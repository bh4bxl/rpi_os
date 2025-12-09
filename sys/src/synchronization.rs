//! Synchronization primitives.

use core::cell::UnsafeCell;

use crate::{exception, state};

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
        exception::asynchronous::exec_with_irq_masked(|| f(data))
    }
}

/// A pseudo-lock that is RW during the single-core kernel init phase and RO afterwards.
pub struct InitStateLock<T>
where
    T: ?Sized,
{
    data: UnsafeCell<T>,
}

impl<T> InitStateLock<T> {
    /// Create an instance.
    pub const fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
        }
    }
}

unsafe impl<T> Send for InitStateLock<T> where T: ?Sized + Send {}
unsafe impl<T> Sync for InitStateLock<T> where T: ?Sized + Send {}

impl<T> interface::ReadWriteEx for InitStateLock<T> {
    type Data = T;

    fn write<'a, R>(&'a self, f: impl FnOnce(&'a mut Self::Data) -> R) -> R {
        assert!(
            state::state_manager().is_init(),
            "InitStateLock::write called after kernel init phase"
        );
        assert!(
            !exception::asynchronous::is_local_irq_masked(),
            "InitStateLock::write called with IRQs unmasked"
        );

        let data = unsafe { &mut *self.data.get() };

        f(data)
    }

    fn read<'a, R>(&'a self, f: impl FnOnce(&'a Self::Data) -> R) -> R {
        let data = unsafe { &*self.data.get() };

        f(data)
    }
}

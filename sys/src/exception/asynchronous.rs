#[cfg(target_arch = "aarch64")]
#[path = "../arch/aarch64/exception/asynchronous.rs"]
mod arch_asynchronous;

use core::marker::PhantomData;

pub use arch_asynchronous::{
    is_local_irq_masked, local_irq_mask, local_irq_mask_save, local_irq_restore, local_irq_unmask,
    print_state,
};

use crate::synchronization::{interface::ReadWriteEx, InitStateLock};

pub type IrqNumber = crate::drivers::arm::IrqNumber;

/// Interrupt descriptor.
#[derive(Copy, Clone)]
pub struct IrqHandlerDescriptor<T>
where
    T: Copy,
{
    /// The IRQ number.
    number: T,

    /// Descriptive name.
    name: &'static str,

    /// Reference to handler trait object.
    handler: &'static (dyn interface::IrqHandler + Sync),
}

impl<T> IrqHandlerDescriptor<T>
where
    T: Copy,
{
    /// Create an instance.
    pub const fn new(
        number: T,
        name: &'static str,
        handler: &'static (dyn interface::IrqHandler + Sync),
    ) -> Self {
        Self {
            number: number,
            name: name,
            handler: handler,
        }
    }

    /// Return the number.
    pub const fn number(&self) -> T {
        self.number
    }

    /// Return the name.
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// Return the handler.
    pub const fn handler(&self) -> &'static (dyn interface::IrqHandler + Sync) {
        self.handler
    }
}

/// IRQContext token.
#[derive(Clone, Copy)]
pub struct IrqContext<'irq_context> {
    _0: PhantomData<&'irq_context ()>,
}

impl<'irq_context> IrqContext<'irq_context> {
    /// Creates an IRQContext token.
    /// # Safety
    #[inline(always)]
    pub unsafe fn new() -> Self {
        IrqContext { _0: PhantomData }
    }
}

/// Asynchronous exception handling interfaces.
pub mod interface {

    /// Implemented by types that handle IRQs.
    pub trait IrqHandler {
        /// Called when the corresponding interrupt is asserted.
        fn handle(&self) -> Result<(), &'static str>;
    }

    /// IRQ management functions.
    pub trait IrqManager {
        /// The IRQ number type depends on the implementation.
        type IrqNumberType: Copy;

        /// Register a handler.
        fn register_handler(
            &self,
            irq_handler_descriptor: super::IrqHandlerDescriptor<Self::IrqNumberType>,
        ) -> Result<(), &'static str>;

        /// Enable an interrupt in the controller.
        fn enable(&self, irq_number: &Self::IrqNumberType);

        /// Handle pending interrupts.
        fn handle_pending_irqs<'irq_context>(
            &'irq_context self,
            ic: &super::IrqContext<'irq_context>,
        );

        /// Print list of registered handlers.
        fn print_handler(&self) {}
    }
}

/// A fake IRQ manager.
struct NullIrqManager;

impl interface::IrqManager for NullIrqManager {
    type IrqNumberType = IrqNumber;

    fn register_handler(
        &self,
        _irq_handler_descriptor: self::IrqHandlerDescriptor<Self::IrqNumberType>,
    ) -> Result<(), &'static str> {
        panic!("No IRQ Manager registered yet",);
    }

    fn enable(&self, _irq_number: &Self::IrqNumberType) {
        panic!("No IRQ Manager registered yet")
    }

    fn handle_pending_irqs<'irq_context>(&'irq_context self, _ic: &self::IrqContext<'irq_context>) {
        panic!("No IRQ Manager registered yet")
    }
}

static NULL_IRQ_MANAGER: NullIrqManager = NullIrqManager {};

/// Executes the provided closure while IRQs are masked on the executing core.
#[inline(always)]
pub fn exec_with_irq_masked<T>(f: impl FnOnce() -> T) -> T {
    let saved = local_irq_mask_save();
    let ret = f();
    local_irq_restore(saved);

    ret
}

static CURR_IRQ_MANAGER: InitStateLock<
    &'static (dyn interface::IrqManager<IrqNumberType = IrqNumber> + Sync),
> = InitStateLock::new(&NULL_IRQ_MANAGER);

/// Register a new IRQ manager.
pub fn register_irq_manager(
    new_manager: &'static (dyn interface::IrqManager<IrqNumberType = IrqNumber> + Sync),
) {
    CURR_IRQ_MANAGER.write(|manager| *manager = new_manager);
}

/// Return a reference to the currently registered IRQ manager.
pub fn irq_manager() -> &'static dyn interface::IrqManager<IrqNumberType = IrqNumber> {
    CURR_IRQ_MANAGER.read(|manager| *manager)
}

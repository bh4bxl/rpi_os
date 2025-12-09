//! GICC Driver - GIC CPU interface.

use tock_registers::{
    interfaces::{Readable, Writeable},
    register_bitfields, register_structs,
    registers::ReadWrite,
};

use crate::{drivers::common::MmioDerefWrapper, exception};

register_bitfields! {
    u32,

    /// CPU Interface Control Register
    CTLR [
        Enable OFFSET(0) NUMBITS(1) []
    ],

    /// Interrupt Priority Mask Register
    PMR [
        Priority OFFSET(0) NUMBITS(8) []
    ],

    /// Interrupt Acknowledge Register
    IAR [
        InterruptID OFFSET(0) NUMBITS(10) []
    ],

    /// End of Interrupt Register
    EOIR [
        EOIINTID OFFSET(0) NUMBITS(10) []
    ],
}

register_structs! {
    pub RegisterBlock {
        (0x000 => ctlr: ReadWrite<u32, CTLR::Register>),
        (0x004 => pmr: ReadWrite<u32, PMR::Register>),
        (0x008 => _reserved1),
        (0x00C => iar: ReadWrite<u32, IAR::Register>),
        (0x010 => eoir: ReadWrite<u32, EOIR::Register>),
        (0x014  => @END),
    }
}

/// Abstraction for the associated MMIO registers.
type Registers = MmioDerefWrapper<RegisterBlock>;

/// Representation of the GIC CPU interface.
pub struct GicC {
    registers: Registers,
}

impl GicC {
    /// Create an instance.
    /// # Safety
    pub const unsafe fn new(mmio_base_addr: usize) -> Self {
        Self {
            registers: Registers::new(mmio_base_addr),
        }
    }

    /// Accept interrupts of any priority.
    /// # Safety
    pub fn priority_accept_all(&self) {
        self.registers.pmr.write(PMR::Priority.val(255));
    }

    /// Enable the interface - start accepting IRQs.
    /// # Safety
    pub fn enable(&self) {
        self.registers.ctlr.write(CTLR::Enable::SET);
    }

    /// Extract the number of the highest-priority pending IRQ.
    pub fn pending_irq_number<'irq_context>(
        &self,
        _ic: &exception::asynchronous::IrqContext<'irq_context>,
    ) -> usize {
        self.registers.iar.read(IAR::InterruptID) as usize
    }

    /// Complete handling of the currently active IRQ.
    /// # Safety
    pub fn mark_comleted<'irq_context>(
        &self,
        irq_number: u32,
        _ic: &exception::asynchronous::IrqContext<'irq_context>,
    ) {
        self.registers.eoir.write(EOIR::EOIINTID.val(irq_number));
    }
}

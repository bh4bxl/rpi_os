//! Architectural asynchronous exception handling.

use core::arch::asm;

use aarch64_cpu::registers::DAIF;
use tock_registers::interfaces::{Readable, Writeable};

mod daif_bits {
    pub const IRQ: u8 = 0b0010;
}

trait DaifField {
    fn daif_field() -> tock_registers::fields::Field<u64, DAIF::Register>;
}

struct Debug;
struct SError;
struct Irq;
struct Fiq;

impl DaifField for Debug {
    fn daif_field() -> tock_registers::fields::Field<u64, DAIF::Register> {
        DAIF::D
    }
}

impl DaifField for SError {
    fn daif_field() -> tock_registers::fields::Field<u64, DAIF::Register> {
        DAIF::A
    }
}

impl DaifField for Irq {
    fn daif_field() -> tock_registers::fields::Field<u64, DAIF::Register> {
        DAIF::I
    }
}

impl DaifField for Fiq {
    fn daif_field() -> tock_registers::fields::Field<u64, DAIF::Register> {
        DAIF::F
    }
}

fn is_masked<T>() -> bool
where
    T: DaifField,
{
    DAIF.is_set(T::daif_field())
}

/// Returns whether IRQs are masked on the executing core.
pub fn is_local_irq_masked() -> bool {
    !is_masked::<Irq>()
}

/// Unmask IRQs on the executing core.
#[inline(always)]
pub fn local_irq_unmask() {
    unsafe {
        asm!("msr DAIFClr, {arg}", arg = const daif_bits::IRQ, options(nomem, nostack, preserves_flags));
    }
}

/// Mask IRQs on the executing core.
#[inline(always)]
pub fn local_irq_mask() {
    unsafe {
        asm!("msr DAIFSet, {arg}", arg = const daif_bits::IRQ, options(nomem, nostack, preserves_flags));
    }
}

/// Mask IRQs on the executing core and return the previously saved interrupt mask bits (DAIF).
#[inline(always)]
pub fn local_irq_mask_save() -> u64 {
    let saved = DAIF.get();
    local_irq_mask();

    saved
}

/// Restore the interrupt mask bits (DAIF) using the callee's argument.
#[inline(always)]
pub fn local_irq_restore(saved: u64) {
    DAIF.set(saved);
}

/// Print the AArch64 exceptions status.
pub fn print_state() {
    use crate::info;

    let to_mask_str = |x| -> _ {
        if x {
            "Masked"
        } else {
            "Unmasked"
        }
    };

    info!("        Debug:  {}", to_mask_str(is_masked::<Debug>()));
    info!("        SError: {}", to_mask_str(is_masked::<SError>()));
    info!("        IRQ:    {}", to_mask_str(is_masked::<Irq>()));
    info!("        FIQ:    {}", to_mask_str(is_masked::<Fiq>()));
}

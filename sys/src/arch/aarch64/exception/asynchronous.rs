//! Architectural asynchronous exception handling.

use aarch64_cpu::registers::DAIF;
use tock_registers::interfaces::Readable;

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

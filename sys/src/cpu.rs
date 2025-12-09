#[path = "arch/boot.rs"]
mod boot;

pub use boot::BOOT_CORE_ID;

#[path = "arch/aarch64/cpu.rs"]
mod arch_cpu;

pub mod smp;

pub use arch_cpu::{nop, wait_forever};

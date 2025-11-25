#[path = "arch/boot.rs"]
mod boot;

#[path = "arch/aarch64/cpu.rs"]
mod arch_cpu;

pub use arch_cpu::{nop, wait_forever};

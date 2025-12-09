#[cfg(target_arch = "aarch64")]
#[path = "aarch64/boot/boot.rs"]
mod arch_boot;

pub use arch_boot::BOOT_CORE_ID;

//#[path = "../drivers/cpu.rs"]
//mod cpu;

#[cfg(target_arch = "aarch64")]
#[path = "arch/aarch64/exception.rs"]
mod arch_exception;

pub mod asynchronous;

pub use arch_exception::{current_privilege_level, handling_init};

/// Privilege levels.
#[derive(Eq, PartialEq)]
pub enum PrivilegeLevel {
    User,
    Kernel,
    Hypervisor,
    TrustZone,
}

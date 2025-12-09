/// The board's physical memory map.
pub mod mmu;

use core::cell::UnsafeCell;

// Symbols from the linker script.
extern "Rust" {
    static __code_start: UnsafeCell<()>;
    static __code_end_exclusive: UnsafeCell<()>;
}

pub(super) mod map {
    /// The inclusive end address of the memory map.
    pub const END_INCLUSIVE: usize = 0xffff_ffff;

    pub const GPIO_OFFSET: usize = 0x0020_0000;
    pub const UART_OFFSET: usize = 0x0020_1000;

    pub mod mmio {
        use super::*;

        pub const BASE: usize = 0xfe00_0000;
        pub const GPIO_BASE: usize = BASE + GPIO_OFFSET;
        pub const UART_BASE: usize = BASE + UART_OFFSET;
        pub const GICD_BASE: usize = 0xff84_1000;
        pub const GICC_BASE: usize = 0xff84_2000;
        pub const END_INCLUSIVE: usize = 0xff84_ffff;
    }
}

/// Start page address of the code segment.
/// # Safety
#[inline(always)]
fn code_start() -> usize {
    unsafe { __code_start.get() as usize }
}

/// Exclusive end page address of the code segment.
/// # Safety
#[inline(always)]
fn code_end_exclusive() -> usize {
    unsafe { __code_end_exclusive.get() as usize }
}

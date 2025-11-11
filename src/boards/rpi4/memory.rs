/// The board's physical memory map.
pub(super) mod map {
    pub const GPIO_OFFSET: u64 = 0x0020_0000;
    pub const UART_OFFSET: u64 = 0x0020_1000;

    pub mod mmio {
        use super::*;

        pub const BASE: u64 = 0xFE00_0000;
        pub const GPIO_BASE: u64 = BASE + GPIO_OFFSET;
        pub const UART_BASE: u64 = BASE + UART_OFFSET;
    }
}

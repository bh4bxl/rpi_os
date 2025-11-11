use core::arch::global_asm;

global_asm!(
    include_str!("boot.S"),
    CONST_CORE_ID_MASK = const 0b11
);

#[unsafe(no_mangle)]
pub unsafe fn _rust_start() -> ! {
    crate::rpi_os_init();
}

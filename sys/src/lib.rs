#![no_std]
#![no_main]

pub mod board;
pub mod common;
pub mod console;
pub mod cpu;
pub mod debug_info;
pub mod driver_manager;
pub mod exception;
pub mod panic;
pub mod print;
pub mod synchronization;
pub mod timer_manager;

// Callbacks for special board.
extern "Rust" {
    fn board_early_init() -> Result<(), &'static str>;

    fn os_early_entry() -> !;
}

/// Early init code
unsafe fn rpi_os_init() -> ! {
    // Board init
    if let Err(x) = board_early_init() {
        panic!("Error initializing board: {}", x);
    }

    // Init all drivers
    driver_manager::driver_manager().init_drivers();

    os_early_entry();
}

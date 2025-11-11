//! The 'kernel' binary

#![no_std]
#![no_main]

use crate::console::console;

mod arch;
mod boards;
mod console;
mod driver_manager;
mod drivers;
mod panic;
mod print;
mod synchronization;

/// Early init code
unsafe fn rpi_os_init() -> ! {
    // Board init
    if let Err(x) = boards::rpi4::board_init() {
        panic!("Error initializing board: {}", x);
    }

    // Init all drivers
    driver_manager::driver_manager().init_drivers();

    rpi_os_main();
}

fn rpi_os_main() -> ! {
    println!(
        "[0] {} version {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );
    println!("[1] Booting on: {}", boards::rpi4::board_name());
    println!("[2] Drivers loaded");
    driver_manager::driver_manager().enumerate();
    println!("[3] Chars written: {}", console::console().chars_written());
    println!("[4] Echoing input now");

    console().clear_rx();
    loop {
        let c = console().read_char();
        console().write_char(c);
    }
}

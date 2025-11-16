//! The 'kernel' binary

#![no_std]
#![no_main]

use core::time::Duration;

use crate::console::console;

mod arch;
mod boards;
mod console;
mod driver_manager;
mod drivers;
mod exception;
mod panic;
mod print;
mod synchronization;
mod timer_manager;

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
    info!(
        "{} version {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );
    info!("Booting on: {}", boards::rpi4::board_name());

    let (_, privilege_level) = exception::current_privilege_level();
    info!("Current privilege level: {}", privilege_level);
    info!("Exception handling state:");
    exception::asynchronous::print_state();

    info!(
        "Architectural timer resolution: {} ns",
        timer_manager::timer_manager().resolution().as_nanos()
    );
    info!("Drivers loaded");
    driver_manager::driver_manager().enumerate();
    info!("Chars written: {}", console::console().chars_written());
    info!("Timer test");

    timer_manager::timer_manager().spin_for(Duration::from_nanos(1));

    for _i in 0..10 {
        info!("Spinning for 1 second");
        timer_manager::timer_manager().spin_for(Duration::from_secs(1));
    }

    info!("Echoing input now");
    console().clear_rx();
    loop {
        let c = console().read_char();
        console().write_char(c);
    }
}

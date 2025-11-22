//! The 'kernel' binary

#![no_std]
#![no_main]

use core::time::Duration;

use crate::console::console;

mod arch;
mod boards;
mod common;
mod console;
mod driver_manager;
mod drivers;
mod exception;
mod memory;
mod panic;
mod print;
mod synchronization;
mod timer_manager;

/// Early init code
unsafe fn rpi_os_init() -> ! {
    use memory::mmu::interface::Mmu;

    if let Err(string) = memory::mmu::mmu().enable_mmu_and_caching() {
        panic!("MMU: {}", string);
    }

    // Board init
    if let Err(x) = boards::rpi4::board_init() {
        panic!("Error initializing board: {}", x);
    }

    // Init all drivers
    driver_manager::driver_manager().init_drivers();

    rpi_os_main();
}

fn rpi_os_main() -> ! {
    use console::interface::Write;
    info!(
        "{} version {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );
    info!("Booting on: {}", boards::rpi4::board_name());

    info!("MMU online. Special regions:");
    boards::rpi4::memory::mmu::virt_mem_layout().print_layout();

    let (_, privilege_level) = exception::current_privilege_level();
    info!("Current privilege level: {}", privilege_level);
    info!("Exception handling state:");
    exception::asynchronous::print_state();

    info!(
        "Architectural timer resolution: {} ns",
        timer_manager::timer_manager().resolution().as_nanos()
    );
    info!("Drivers loaded:");
    driver_manager::driver_manager().enumerate();
    info!("Chars written: {}", console::console().chars_written());

    info!("Timer test, 1s");
    timer_manager::timer_manager().spin_for(Duration::from_secs(1));
    info!("Timer test OK");

    let remapped_uart = unsafe { drivers::serial::pl011_uart::Pl011Uart::new(0x1fff_1000) };
    writeln!(
        remapped_uart,
        "[     !!!    ] Writing through the remapped UART at 0x1FFF_1000"
    )
    .unwrap();

    info!("Echoing input now");
    console().clear_rx();
    loop {
        let c = console().read_char();
        console().write_char(c);
    }
}

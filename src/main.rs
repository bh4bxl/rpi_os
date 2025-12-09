//! The 'kernel' binary

#![no_std]
#![no_main]

use core::time::Duration;

use ros_sys::{board, cpu, driver_manager, exception, info, timer_manager};

mod boards;
mod drivers;
mod memory;

#[no_mangle]
unsafe fn board_early_init() -> Result<(), &'static str> {
    use memory::mmu::interface::Mmu;

    exception::handling_init();

    if let Err(str) = memory::mmu::mmu().enable_mmu_and_caching() {
        panic!("MMU: {}", str);
    }

    boards::rpi4::board_init()
}

#[no_mangle]
fn os_early_entry() -> ! {
    info!(
        "{} version {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );
    info!("Booting on: {}", board::board().board_name());

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
    info!(
        "Chars written: {}",
        ros_sys::console::console().chars_written()
    );

    info!("Registered IRQ handlers:");
    exception::asynchronous::irq_manager().print_handler();

    info!("Timer test, 1s");
    timer_manager::timer_manager().spin_for(Duration::from_secs(1));
    info!("Timer test OK");

    info!("Echoing input now");
    cpu::wait_forever()
}

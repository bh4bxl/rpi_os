use core::sync::atomic::{AtomicBool, Ordering};

use crate::{
    boards::rpi4::memory::map::mmio,
    console::register_console,
    driver_manager,
    drivers::{
        self,
        gpio::{interface::Gpio, GpioPupPdn},
        serial::interface::Uart,
    },
};

pub mod memory;

static GPIO: drivers::gpio::bcm2711_gpio::Bcm2711Gpio =
    unsafe { drivers::gpio::bcm2711_gpio::Bcm2711Gpio::new(mmio::GPIO_BASE) };

static PL011_UART: drivers::serial::pl011_uart::Pl011Uart =
    unsafe { drivers::serial::pl011_uart::Pl011Uart::new(mmio::UART_BASE) };

fn gpio_config() -> Result<(), &'static str> {
    // Pin 14, 15 -> uart func, pull-up
    GPIO.set_func(14, 0);
    GPIO.set_func(15, 0);
    GPIO.set_pup_pdn(14, GpioPupPdn::PullUp);
    GPIO.set_pup_pdn(15, GpioPupPdn::PullUp);
    Ok(())
}

fn init_gpio() -> Result<(), &'static str> {
    let gpio_desc = driver_manager::DeviceDriverDescriptor::new(&GPIO, Some(gpio_config));
    driver_manager::driver_manager().register_driver(gpio_desc);

    Ok(())
}

fn uart_config() -> Result<(), &'static str> {
    PL011_UART.set_baud(115200);

    Ok(())
}

fn init_uart() -> Result<(), &'static str> {
    let uart_desc = driver_manager::DeviceDriverDescriptor::new(&PL011_UART, Some(uart_config));
    driver_manager::driver_manager().register_driver(uart_desc);

    Ok(())
}

pub unsafe fn board_init() -> Result<(), &'static str> {
    static INIT_DONE: AtomicBool = AtomicBool::new(false);
    if INIT_DONE.load(Ordering::Relaxed) {
        return Err("Init already done");
    }

    init_gpio()?;

    init_uart()?;

    register_console(&PL011_UART);

    INIT_DONE.store(true, Ordering::Relaxed);
    Ok(())
}

//pub fn console() -> &'static dyn console::interface::All {
//    &PL011_UART
//}

pub fn board_name() -> &'static str {
    "Raspberry Pi 4"
}

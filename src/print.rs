use core::fmt;

use crate::console;

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    console::console().write_fmt(args).unwrap();
}

/// Prints without a newline.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::print::_print(format_args!($($arg)*)));
}

/// Prints with a newline.
#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n")
    };
    ($fmt:expr) => {
        $crate::print!(concat!($fmt, "\n"))
    };
    ($fmt:expr, $($arg:tt)*) => ($crate::print!(concat!($fmt, "\n"), $($arg)*));
}

/// Prints an info, with a newline.
#[macro_export]
macro_rules! info {
    () => {
        let timestamp = $crate::timer_manager::timer_manager().uptime();
        $crate::println!(
            "[I {:>3}.{:06}] ",
            timestamp.as_secs(),
            timestamp.subsec_micros(),
        );
    };
    ($fmt:expr) => {
        let timestamp = $crate::timer_manager::timer_manager().uptime();
        $crate::println!(
            "[I {:>3}.{:06}] {}",
            timestamp.as_secs(),
            timestamp.subsec_micros(),
            $fmt
        );
    };
    ($fmt:expr, $($arg:tt)*) => {
        let timestamp = $crate::timer_manager::timer_manager().uptime();
        ($crate::println!(concat!("[I {:>3}.{:06}] ", $fmt),
            timestamp.as_secs(),
            timestamp.subsec_micros(),
            $($arg)*));
    };
}

/// Prints a warning, with a newline.
#[macro_export]
macro_rules! warn {
    () => {
        let timestamp = $crate::timer_manager::timer_manager().uptime();
        $crate::println!(
            "[W {:>3}.{:06}] ",
            timestamp.as_secs(),
            timestamp.subsec_micros(),
        );
    };
    ($fmt:expr) => {
        let timestamp = $crate::timer_manager::timer_manager().uptime();
        $crate::println!(
            "[W {:>3}.{:06}] {}",
            timestamp.as_secs(),
            timestamp.subsec_micros(),
            $fmt
        );
    };
    ($fmt:expr, $($arg:tt)*) => {
        let timestamp = $crate::timer_manager::timer_manager().uptime();
        ($crate::println!(concat!("[W {:>3}.{:06}] ", $fmt),
            timestamp.as_secs(),
            timestamp.subsec_micros(),
            $($arg)*));
    };
}

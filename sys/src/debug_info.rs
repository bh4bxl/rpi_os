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

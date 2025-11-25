use crate::synchronization::{interface::Mutex, NullLock};

/// Console interfaces.
pub mod interface {
    use core::fmt;

    /// Console write functions.
    pub trait Write {
        /// Write a single character.
        fn write_char(&self, c: char);

        /// Write a Rust format string.
        fn write_fmt(&self, args: fmt::Arguments) -> fmt::Result;

        /// Block until the last buffered character has been physically put on the TX wire.
        fn flush(&self);
    }

    /// Console read functions.
    pub trait Read {
        /// Read a single character.
        fn read_char(&self) -> char {
            ' '
        }

        /// Clear RX buffers, if any.
        fn clear_rx(&self);
    }

    /// Console statistics.
    pub trait Statistics {
        /// Return the number of characters written.
        fn chars_written(&self) -> usize {
            0
        }

        /// Return the number of characters read.
        fn chars_read(&self) -> usize {
            0
        }
    }

    /// Trait alias for a full-fledged console.
    pub trait All: Write + Read + Statistics {}
}

/// A placeholder.
struct NullConsole;

impl interface::Write for NullConsole {
    fn write_char(&self, _c: char) {}

    fn write_fmt(&self, _args: core::fmt::Arguments) -> core::fmt::Result {
        core::fmt::Result::Ok(())
    }

    fn flush(&self) {}
}

impl interface::Read for NullConsole {
    fn clear_rx(&self) {}
}

impl interface::Statistics for NullConsole {}

impl interface::All for NullConsole {}

static NULL_CONSOLE: NullConsole = NullConsole {};

static CURR_CONSOLE: NullLock<&'static (dyn interface::All + Sync)> = NullLock::new(&NULL_CONSOLE);

/// Register a new console.
pub fn register_console(new_console: &'static (dyn interface::All + Sync)) {
    CURR_CONSOLE.lock(|con| *con = new_console);
}

/// Return a reference to the console.
pub fn console() -> &'static dyn interface::All {
    CURR_CONSOLE.lock(|con| *con)
}

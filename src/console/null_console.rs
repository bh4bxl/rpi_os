use core::fmt;

use crate::console::interface;

pub struct NullConsole;

pub static NULL_CONSOLE: NullConsole = NullConsole {};

impl interface::Write for NullConsole {
    fn write_char(&self, _c: char) {}

    fn write_fmt(&self, _args: core::fmt::Arguments) -> core::fmt::Result {
        fmt::Result::Ok(())
    }

    fn flush(&self) {}
}

impl interface::Read for NullConsole {
    fn clear_rx(&self) {}
}

impl interface::Statistics for NullConsole {}

impl interface::All for NullConsole {}

//! Timer primitives.

use core::time::Duration;

#[path = "arch/aarch64/timer.rs"]
mod arch_timer;

/// Provides time management functions.
pub struct TimerManager;

impl TimerManager {
    /// Create an instance.
    pub const fn new() -> Self {
        Self
    }

    /// The timer's resolution.
    pub fn resolution(&self) -> Duration {
        arch_timer::resolution()
    }

    /// The uptime since power-on of the device.
    pub fn uptime(&self) -> Duration {
        arch_timer::uptime()
    }

    /// Spin for a given duration.
    pub fn spin_for(&self, duration: Duration) {
        arch_timer::spin_for(duration);
    }
}

static TIMER_MANAGER: TimerManager = TimerManager::new();

/// Return a reference to the global TimeManager.
pub fn timer_manager() -> &'static TimerManager {
    &TIMER_MANAGER
}

pub mod bcm2711_gpio;

#[allow(unused)]
pub enum GpioDirect {
    In,
    Out,
}

#[allow(unused)]
pub enum GpioLevel {
    Low,
    High,
}

#[allow(unused)]
pub enum GpioPupPdn {
    Off,
    PullUp,
    PullDown,
}

#[allow(dead_code)]
pub mod interface {
    use crate::drivers::gpio::{GpioDirect, GpioLevel, GpioPupPdn};

    pub trait Gpio {
        fn set_direct(&self, pin: usize, io: GpioDirect);

        fn set_level(&self, pin: usize, level: GpioLevel);

        fn set_pup_pdn(&self, pin: usize, pup_pdn: GpioPupPdn);

        fn set_func(&self, pin: usize, func: u8);
    }
}

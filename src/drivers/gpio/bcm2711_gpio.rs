use aarch64_cpu::registers::{Readable, Writeable};
use tock_registers::{
    register_bitfields, register_structs,
    registers::{ReadWrite, WriteOnly},
};

use ros_sys::synchronization::{interface::Mutex, NullLock};

use crate::{
    driver_manager::interface::DeviceDriver,
    drivers::{
        common::MmioDerefWrapper,
        gpio::{interface, GpioDirect, GpioLevel, GpioPupPdn},
    },
    //synchronization::{interface::Mutex, NullLock},
};

// GPIO registers.
register_bitfields! [
    u32,

    GPFSEL [
        /// GPIO Fucntion Select
        FSEL9 OFFSET(27) NUMBITS(3) [],
        FSEL8 OFFSET(24) NUMBITS(3) [],
        FSEL7 OFFSET(21) NUMBITS(3) [],
        FSEL6 OFFSET(18) NUMBITS(3) [],
        FSEL5 OFFSET(15) NUMBITS(3) [],
        FSEL4 OFFSET(12) NUMBITS(3) [],
        FSEL3 OFFSET(9) NUMBITS(3) [],
        FSEL2 OFFSET(6) NUMBITS(3) [],
        FSEL1 OFFSET(3) NUMBITS(3) [],
        FSEL0 OFFSET(0) NUMBITS(3) []
    ],

    GPSET [
        SET OFFSET(0) NUMBITS(32) []
    ],

    GPCLR [
        CLR OFFSET(0) NUMBITS(32) []
    ],

    GPLEV[
        LEV OFFSET(0) NUMBITS(32) []
    ],

    GPPUPPDN [
        GPIO_PUP_PDN_CNTRL15 OFFSET(30) NUMBITS(2) [],
        GPIO_PUP_PDN_CNTRL14 OFFSET(28) NUMBITS(2) [],
        GPIO_PUP_PDN_CNTRL13 OFFSET(26) NUMBITS(2) [],
        GPIO_PUP_PDN_CNTRL12 OFFSET(24) NUMBITS(2) [],
        GPIO_PUP_PDN_CNTRL11 OFFSET(22) NUMBITS(2) [],
        GPIO_PUP_PDN_CNTRL10 OFFSET(20) NUMBITS(2) [],
        GPIO_PUP_PDN_CNTRL09 OFFSET(18) NUMBITS(2) [],
        GPIO_PUP_PDN_CNTRL08 OFFSET(16) NUMBITS(2) [],
        GPIO_PUP_PDN_CNTRL07 OFFSET(14) NUMBITS(2) [],
        GPIO_PUP_PDN_CNTRL06 OFFSET(12) NUMBITS(2) [],
        GPIO_PUP_PDN_CNTRL05 OFFSET(10) NUMBITS(2) [],
        GPIO_PUP_PDN_CNTRL04 OFFSET(8) NUMBITS(2) [],
        GPIO_PUP_PDN_CNTRL03 OFFSET(6) NUMBITS(2) [],
        GPIO_PUP_PDN_CNTRL02 OFFSET(4) NUMBITS(2) [],
        GPIO_PUP_PDN_CNTRL01 OFFSET(2) NUMBITS(2) [],
        GPIO_PUP_PDN_CNTRL00 OFFSET(0) NUMBITS(2) []
    ],
];

register_structs! {
    RegisterBlock {
        (0x00 => gpfsel: [ReadWrite<u32, GPFSEL::Register>; 6]),
        (0x18 => _reserved1),
        (0x1c => gpset: [WriteOnly<u32, GPSET::Register>; 2]),
        (0x24 => _reserved2),
        (0x28 => gpclr: [WriteOnly<u32, GPCLR::Register>; 2]),
        (0x30 => _reserved3),
        (0x34 => gplev: [WriteOnly<u32, GPCLR::Register>; 2]),
        (0x3c => _reserved4),
        (0xe4 => gppuppdn: [ReadWrite<u32, GPPUPPDN::Register>; 4]),
        (0xf4 => _reserved6),
        (0xfc => @END),
    }
}

/// Abstraction for the associated MMIO registers.
type Registers = MmioDerefWrapper<RegisterBlock>;

struct Bcm2711GpioInner {
    register: Registers,
}

impl Bcm2711GpioInner {
    /// Create an instance.
    pub const fn new(base_addr: usize) -> Self {
        Self {
            register: Registers::new(base_addr),
        }
    }

    fn set_func_reg(&self, pin: usize, alt_func: u32) {
        if pin > 57 {
            return;
        }
        fn fsel_index(pin: usize) -> (usize, usize) {
            (pin / 10, (pin % 10) * 3)
        }

        let (reg, shift) = fsel_index(pin);
        let mut v = self.register.gpfsel[reg].get();
        v &= !(0b111 << shift);
        v |= (alt_func & 0b111) << shift;
        self.register.gpfsel[reg].set(v);
    }
}

impl interface::Gpio for Bcm2711GpioInner {
    fn set_direct(&self, pin: usize, io: GpioDirect) {
        let alt_func = match io {
            GpioDirect::In => 0x000,
            GpioDirect::Out => 0x001,
        };

        self.set_func_reg(pin, alt_func);
    }

    fn set_level(&self, _pin: usize, _level: GpioLevel) {}

    fn set_pup_pdn(&self, pin: usize, pup_pdn: GpioPupPdn) {
        if pin > 57 {
            return;
        }
        let pud = match pup_pdn {
            GpioPupPdn::Off => 0b00,
            GpioPupPdn::PullUp => 0b01,
            GpioPupPdn::PullDown => 0b10,
        };

        fn pud_index(pin: usize) -> (usize, usize) {
            (pin / 16, (pin % 16) * 2)
        }

        let (reg, shift) = pud_index(pin);
        let mut v = self.register.gppuppdn[reg].get();
        v &= !(0b11 << shift);
        v |= (pud & 0b11) << shift;
        self.register.gppuppdn[reg].set(v);
    }

    fn set_func(&self, pin: usize, func: u8) {
        if func > 5 {
            return;
        }
        let alt_func = match func {
            1 => 0b101,
            2 => 0b110,
            3 => 0b111,
            4 => 0b011,
            5 => 0b010,
            _ => 0b100,
        };

        self.set_func_reg(pin, alt_func);
    }
}

/// Representation of the GPIO HW.
pub struct Bcm2711Gpio {
    inner: NullLock<Bcm2711GpioInner>,
}

impl Bcm2711Gpio {
    pub const COMPATIBLE: &'static str = "BCM2711 GPIO";

    /// Create an instance.
    /// # Safety
    pub const unsafe fn new(mmio_base_addr: usize) -> Self {
        Self {
            inner: NullLock::new(Bcm2711GpioInner::new(mmio_base_addr)),
        }
    }
}

impl interface::Gpio for Bcm2711Gpio {
    fn set_direct(&self, pin: usize, io: super::GpioDirect) {
        self.inner.lock(|inner| inner.set_direct(pin, io));
    }
    fn set_level(&self, pin: usize, level: super::GpioLevel) {
        self.inner.lock(|inner| inner.set_level(pin, level));
    }
    fn set_pup_pdn(&self, pin: usize, pup_pdn: super::GpioPupPdn) {
        self.inner.lock(|inner| inner.set_pup_pdn(pin, pup_pdn));
    }
    fn set_func(&self, pin: usize, func: u8) {
        self.inner.lock(|inner| inner.set_func(pin, func));
    }
}

impl DeviceDriver for Bcm2711Gpio {
    fn compatible(&self) -> &'static str {
        Self::COMPATIBLE
    }
}

use core::fmt::{self, Write};

use aarch64_cpu::registers::{ReadWriteable, Readable, Writeable};
use tock_registers::{
    register_bitfields, register_structs,
    registers::{ReadOnly, ReadWrite, WriteOnly},
};

use ros_sys::{console, cpu, drivers::common::MmioDerefWrapper, exception};

use crate::{driver_manager, drivers::serial::interface};

use ros_sys::synchronization::{interface::Mutex, IrqSafeNullLock};

pub const UART_CLOCK: u32 = 48_000_000;

register_bitfields![
    u32,

    DR [
        DATA OFFSET(0) NUMBITS(8) []
    ],

    RSRECR [
        FE OFFSET(0) NUMBITS(1) [], // Framing Error
        PE OFFSET(1) NUMBITS(1) [], // Parity Error
        BE OFFSET(2) NUMBITS(1) [], // Break Error
        OE OFFSET(3) NUMBITS(1) []  // Overrun Error
    ],

    FR [
        CTS OFFSET(0) NUMBITS(1) [],
        DSR OFFSET(1) NUMBITS(1) [],
        DCD OFFSET(2) NUMBITS(1) [],
        BUSY OFFSET(3) NUMBITS(1) [],
        RXFE OFFSET(4) NUMBITS(1) [],   // Receive FIFO Empty
        TXFF OFFSET(5) NUMBITS(1) [],   // Transmit FIFO Full
        RXFF OFFSET(6) NUMBITS(1) [],   // Receive FIFO Full
        TXFE OFFSET(7) NUMBITS(1) [],   // Transmit FIFO Empty
        RI OFFSET(8) NUMBITS(1) []
    ],

    IBRD [
        BAUD_DIVINT OFFSET(0) NUMBITS(16) []
    ],

    FBRD [
        BAUD_DIVFRAC OFFSET(0) NUMBITS(6) []
    ],

    LCRH [
        BRK OFFSET(0) NUMBITS(1) [],
        PEN OFFSET(1) NUMBITS(1) [],
        EPS OFFSET(2) NUMBITS(1) [],
        STP2 OFFSET(3) NUMBITS(1) [],
        FEN OFFSET(4) NUMBITS(1) [      // FIFO Enable
            FifosDisabled = 0,
            FifosEnabled = 1,
        ],
        WLEN OFFSET(5) NUMBITS(2) [     // Word Length
            FiveBits = 0b00,
            SixBits = 0b01,
            SevenBits = 0b10,
            EightBits = 0b11
        ],
        SPS   OFFSET(7) NUMBITS(1) []   // Stick Parity Select
    ],

    CR [
        UARTEN OFFSET(0) NUMBITS(1) [],
        SIREN OFFSET(1) NUMBITS(1) [],
        SIRLP OFFSET(2) NUMBITS(1) [],
        // Reserved [3..6]
        LBE OFFSET(7) NUMBITS(1) [],    // Loopback Enable
        TXE OFFSET(8) NUMBITS(1) [],    // Transmit Enable
        RXE OFFSET(9) NUMBITS(1) [],    // Receive Enable
        DTR OFFSET(10) NUMBITS(1) [],
        RTS OFFSET(11) NUMBITS(1) [],
        Out1 OFFSET(12) NUMBITS(1) [],
        Out2 OFFSET(13) NUMBITS(1) [],
        RTSEN OFFSET(14) NUMBITS(1) [],
        CTSEN OFFSET(15) NUMBITS(1) []
    ],

    IFLS [
        TXIFLSEL OFFSET(0) NUMBITS(3) [],
        RXIFLSEL OFFSET(3) NUMBITS(3) [
            OneEigth = 0b000,
            OneQuarter = 0b001,
            OneHalf = 0b010,
            ThreeQuarters = 0b011,
            SevenEights = 0b100,
        ]
    ],

    IMSC [
        // Mask Set/Clear Interrupt bits (1 = enabled)
        RXIM OFFSET(4) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1,
        ],
        TXIM OFFSET(5) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1,
        ],
        RTIM OFFSET(6) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1,
        ],

    ],

    MIS [
        RXMIS OFFSET(4) NUMBITS(1) [],
        RTMIS OFFSET(6) NUMBITS(1) [],
    ],

    ICR [
        ALL OFFSET(0) NUMBITS(11) [],
    ]
];

register_structs! {
    RegisterBlock {
        (0x00 => dr: ReadWrite<u32, DR::Register>),
        (0x04 => rsrecr: ReadWrite<u32, RSRECR::Register>),
        (0x08 => _reserved0),
        (0x18 => fr: ReadOnly<u32, FR::Register>),
        (0x1C => _reserved1),
        (0x20 => ilpr: ReadWrite<u32>),
        (0x24 => ibrd: ReadWrite<u32, IBRD::Register>),
        (0x28 => fbrd: ReadWrite<u32, FBRD::Register>),
        (0x2C => lcrh: ReadWrite<u32, LCRH::Register>),
        (0x30 => cr: ReadWrite<u32, CR::Register>),
        (0x34 => ifls: ReadWrite<u32, IFLS::Register>),
        (0x38 => imsc: ReadWrite<u32, IMSC::Register>),
        (0x3C => ris: ReadOnly<u32>),
        (0x40 => mis: ReadOnly<u32, MIS::Register>),
        (0x44 => icr: WriteOnly<u32, ICR::Register>),
        (0x48 => dmacr: ReadWrite<u32>),
        (0x4C => _reserved2),
        (0x80 => @END),
    }
}

/// Abstraction for the associated MMIO registers.
type Registers = MmioDerefWrapper<RegisterBlock>;

#[derive(PartialEq)]
enum BlockingMode {
    Blocking,
    NonBlocking,
}

struct Pl011UartInner {
    registers: Registers,
    chars_written: usize,
    chars_read: usize,
}

impl Pl011UartInner {
    /// Create an instance.
    /// # Safety
    pub const unsafe fn new(base_addr: usize) -> Self {
        Self {
            registers: Registers::new(base_addr),
            chars_written: 0,
            chars_read: 0,
        }
    }

    /// Block execution until the last buffered character has been physically put on the TX wire.
    fn flush(&self) {
        while self.registers.fr.matches_all(FR::BUSY::SET) {
            cpu::nop();
        }
    }

    /// Set up baud rate and characteristics.
    pub fn set_baud(&self, baud: u32) {
        self.flush();

        // 1. Disable UART
        self.registers.cr.modify(CR::UARTEN::CLEAR);

        // 2. Clear interrupts
        self.registers.icr.write(ICR::ALL::CLEAR);

        // 3. Baud divisor calculation
        let ibrd = UART_CLOCK / (16 * baud);
        let fbrd = ((UART_CLOCK % (16 * baud)) * 64 + baud / 2) / baud;

        self.registers.ibrd.set(ibrd);
        self.registers.fbrd.set(fbrd);

        // 4. Line control: 8 bits, FIFO enabled
        // Set WLEN = 3 (8 bits), FEN = 1
        self.registers
            .lcrh
            .modify(LCRH::WLEN::EightBits + LCRH::FEN::FifosEnabled);

        // Set RX FIFO fill level at 1/8.
        self.registers.ifls.write(IFLS::RXIFLSEL::OneEigth);

        // Enable RX IRQ + RX timeout IRQ.
        self.registers
            .imsc
            .write(IMSC::RXIM::Enabled + IMSC::RTIM::Enabled);

        // 5. Enable UART, TX and RX
        self.registers
            .cr
            .modify(CR::UARTEN::SET + CR::TXE::SET + CR::RXE::SET);
    }

    /// Send a character.
    fn write_char(&mut self, c: char) {
        while self.registers.fr.matches_all(FR::TXFF::SET) {
            cpu::nop();
        }

        self.registers.dr.set(c as u32);

        self.chars_written += 1;
    }

    /// Retrieve a character.
    fn read_char_converting(&mut self, blocking_mode: BlockingMode) -> Option<char> {
        if self.registers.fr.matches_all(FR::RXFE::SET) {
            if blocking_mode == BlockingMode::NonBlocking {
                return None;
            }

            while self.registers.fr.matches_all(FR::RXFE::SET) {
                cpu::nop();
            }
        }

        let mut ret = self.registers.dr.get() as u8 as char;

        if ret == '\r' {
            ret = '\n';
        }

        self.chars_read += 1;

        Some(ret)
    }
}

/// Implementing `core::fmt::Write`
impl fmt::Write for Pl011UartInner {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.write_char(c);
        }

        Ok(())
    }
}

pub struct Pl011Uart {
    inner: IrqSafeNullLock<Pl011UartInner>,
}

impl Pl011Uart {
    pub const COMPATIBLE: &'static str = "PL011 Uart";

    /// Create an instance.
    pub const unsafe fn new(mmio_base_addr: usize) -> Self {
        Self {
            inner: IrqSafeNullLock::new(Pl011UartInner::new(mmio_base_addr)),
        }
    }
}

impl interface::Uart for Pl011Uart {
    fn set_baud(&self, baud: u32) {
        self.inner.lock(|inner| inner.set_baud(baud));
    }
}

impl driver_manager::interface::DeviceDriver for Pl011Uart {
    type IrqNumberType = exception::asynchronous::IrqNumber;

    fn compatible(&self) -> &'static str {
        Self::COMPATIBLE
    }

    fn register_and_enable_irq_handler(
        &'static self,
        irq_number: &Self::IrqNumberType,
    ) -> Result<(), &'static str> {
        use exception::asynchronous::{irq_manager, IrqHandlerDescriptor};

        let descriptor = IrqHandlerDescriptor::new(*irq_number, Self::COMPATIBLE, self);

        irq_manager().register_handler(descriptor)?;
        irq_manager().enable(irq_number);

        Ok(())
    }
}

impl console::interface::Write for Pl011Uart {
    fn write_char(&self, c: char) {
        self.inner.lock(|inner| inner.write_char(c));
    }

    fn write_fmt(&self, args: fmt::Arguments) -> fmt::Result {
        self.inner.lock(|inner| inner.write_fmt(args))
    }

    fn flush(&self) {
        self.inner.lock(|inner| inner.flush());
    }
}

impl console::interface::Read for Pl011Uart {
    fn read_char(&self) -> char {
        self.inner
            .lock(|inner| inner.read_char_converting(BlockingMode::Blocking).unwrap())
    }

    fn clear_rx(&self) {
        while self
            .inner
            .lock(|inner| inner.read_char_converting(BlockingMode::NonBlocking))
            .is_some()
        {}
    }
}

impl console::interface::Statistics for Pl011Uart {
    fn chars_written(&self) -> usize {
        self.inner.lock(|inner| inner.chars_written)
    }

    fn chars_read(&self) -> usize {
        self.inner.lock(|inner| inner.chars_read)
    }
}

impl console::interface::All for Pl011Uart {}

impl exception::asynchronous::interface::IrqHandler for Pl011Uart {
    fn handle(&self) -> Result<(), &'static str> {
        self.inner.lock(|inner| {
            let pending = inner.registers.mis.extract();

            // Clear all pending IRQs.
            inner.registers.icr.write(ICR::ALL::CLEAR);

            // Check for any kind of RX interrupt.
            if pending.matches_any(&[MIS::RXMIS::SET, MIS::RTMIS::SET]) {
                // Echo any received characters.
                while let Some(c) = inner.read_char_converting(BlockingMode::NonBlocking) {
                    inner.write_char(c);
                }
            }
        });

        Ok(())
    }
}

//! GICv2 Driver - ARM Generic Interrupt Controller v2.

use crate::{
    driver_manager,
    drivers::common::BoundedUsize,
    exception, info,
    synchronization::{interface::ReadWriteEx, InitStateLock},
};

mod gicc;
mod gicd;

type HandlerTable = [Option<exception::asynchronous::IrqHandlerDescriptor<IrqNumber>>;
    IrqNumber::MAX_INCLUSIVE + 1];

pub type IrqNumber = BoundedUsize<{ GicV2::MAX_IRQ_NUMBER }>;

// Representation of the GIC.
pub struct GicV2 {
    /// The Distributor.
    gicd: gicd::GicD,

    /// The CPU Interface.
    gicc: gicc::GicC,

    /// Stores registered IRQ handlers. Writable only during kernel init. RO afterwards.
    handler_table: InitStateLock<HandlerTable>,
}

impl GicV2 {
    const MAX_IRQ_NUMBER: usize = 300;

    pub const COMPATIBLE: &'static str = "GICv2 (ARM Generic Interrupt Controller v2)";

    /// Create an instance.
    /// # Safety
    pub const unsafe fn new(gicd_mmio_base_addr: usize, gicc_mmio_base_addr: usize) -> Self {
        Self {
            gicd: gicd::GicD::new(gicd_mmio_base_addr),
            gicc: gicc::GicC::new(gicc_mmio_base_addr),
            handler_table: InitStateLock::new([None; IrqNumber::MAX_INCLUSIVE + 1]),
        }
    }
}

impl driver_manager::interface::DeviceDriver for GicV2 {
    type IrqNumberType = IrqNumber;

    fn compatible(&self) -> &'static str {
        Self::COMPATIBLE
    }

    unsafe fn init(&self) -> Result<(), &'static str> {
        if crate::cpu::BOOT_CORE_ID == crate::cpu::smp::core_id() {
            self.gicd.boot_core_init();
        }

        self.gicc.priority_accept_all();
        self.gicc.enable();

        Ok(())
    }
}

impl exception::asynchronous::interface::IrqManager for GicV2 {
    type IrqNumberType = IrqNumber;

    fn register_handler(
        &self,
        irq_handler_descriptor: exception::asynchronous::IrqHandlerDescriptor<Self::IrqNumberType>,
    ) -> Result<(), &'static str> {
        self.handler_table.write(|table| {
            let irq_number = irq_handler_descriptor.number().get();

            if table[irq_number].is_some() {
                return Err("IRQ handler already registered");
            }

            table[irq_number] = Some(irq_handler_descriptor);

            Ok(())
        })
    }

    fn enable(&self, irq_number: &Self::IrqNumberType) {
        self.gicd.enable(irq_number);
    }

    fn handle_pending_irqs<'irq_context>(
        &'irq_context self,
        ic: &exception::asynchronous::IrqContext<'irq_context>,
    ) {
        // Extract the highest priority pending IRQ number from the IAR.
        let irq_number = self.gicc.pending_irq_number(ic);

        if irq_number > GicV2::MAX_IRQ_NUMBER {
            return;
        }

        // Call the IRQ handler. Panic if there is none.
        self.handler_table.read(|table| {
            match table[irq_number] {
                None => panic!("No handler registered for IRQ {}", irq_number),
                Some(descriptor) => {
                    // Call the IRQ handler. Panics on failure.
                    descriptor.handler().handle().expect("Error handling IRQ");
                }
            }
        });

        // Signal completion of handling.
        self.gicc.mark_comleted(irq_number as u32, ic);
    }

    fn print_handler(&self) {
        use crate::info;

        info!("    Peripheral handler:");

        self.handler_table.read(|table| {
            for (i, opt) in table.iter().skip(32).enumerate() {
                if let Some(handler) = opt {
                    info!("        {: >3}. {}", i + 32, handler.name());
                }
            }
        });
    }
}

pub mod pl011_uart;

pub mod interface {
    pub trait Uart {
        fn set_baud(&self, baud: u32);
    }
}

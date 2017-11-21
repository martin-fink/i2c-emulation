mod error;
mod bit_layer;
mod rw_bit;
mod pin_thread;

pub use self::error::Error;
pub use self::rw_bit::RWBit;
pub use self::pin_thread::PinThread;

pub trait I2CProtocol {
    /// Checks if the received address is our address
    /// The addresses most significant bit is always 0, since we only support
    /// 7-bit addressing mode and the R/W-bit is stripped.
    fn check_address(&self, address: u8) -> bool;

    fn set_register(&mut self, register: usize, data: u8) -> bool;

    fn get_register(&self, register: usize) -> Option<u8>;

    /// Signals an error to the upper layer
    fn on_error(&mut self, error: Error);
}

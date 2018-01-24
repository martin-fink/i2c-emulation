mod error;
mod bit_layer;
mod rw_bit;
mod pin_thread;

pub use self::bit_layer::BitLayer;
pub use self::error::Error;
use self::rw_bit::RWBit;
use self::pin_thread::PinThread;

pub trait I2CProtocol {
    /// Checks if the received address is our address
    /// The addresses most significant bit is always 0, since we only support
    /// 7-bit addressing mode and the R/W-bit is stripped.
    fn check_address(&self, address: u8) -> bool;

    /// Sets a register
    /// Returns if the register does exist
    fn set_register(&mut self, register: usize, data: u8);

    /// Gets the register
    /// Returns Option::None if the register does not exist
    fn get_register(&self, register: usize) -> u8;
}

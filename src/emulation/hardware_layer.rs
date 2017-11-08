use super::pin::{I2CPin, PinError};
use std::error::Error;
use std::convert;
use std::fmt;

#[derive(Debug)]
pub struct HardwareError {
    message: String,
}

pub enum ReadType {
    Start,
    Stop,
    Data(u8),
}

impl Error for HardwareError {
    fn description(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for HardwareError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.message.fmt(f)
    }
}

impl convert::From<PinError> for HardwareError {
    fn from(error: PinError) -> Self {
        HardwareError {
            message: String::from(error.description()),
        }
    }
}

pub struct HardwareLayer {
    scl: I2CPin,
    sda: I2CPin,
}

impl HardwareLayer {
    pub fn new(sda_pin: u8, scl_pin: u8) -> Self {
        HardwareLayer {
            sda: I2CPin::new(sda_pin),
            scl: I2CPin::new(scl_pin),
        }
    }

    pub fn write(&mut self, data: u8) -> Result<(), HardwareError> {
        unimplemented!()
    }

    pub fn read(&self) -> Result<ReadType, HardwareError> {
        unimplemented!()
    }

    pub fn ack(&mut self) -> Result<(), HardwareError> {
        unimplemented!()
    }
}

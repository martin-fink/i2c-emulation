use super::pin::{I2CPin, PinError};
use std::error::Error;
use std::convert;
use std::fmt;

#[derive(Debug)]
pub struct HardwareError {
    message: String,
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

pub struct BitLayer {
    scl: I2CPin,
    sda: I2CPin,
}

impl BitLayer {
    pub fn new(sda_pin: u8, scl_pin: u8) -> Self {
        BitLayer {
            sda: I2CPin::new(sda_pin),
            scl: I2CPin::new(scl_pin),
        }
    }

    pub fn wait_for_start(&mut self) -> Result<(), HardwareError> {
        self.sda.wait_until(false)?;
        if self.scl.read()? == 0 {
            Err(HardwareError {
                message: String::from("scl was low, expected start.")
            })
        } else {
            Ok(())
        }
    }

    pub fn write_byte(&mut self, data: u8) -> Result<(), HardwareError> {
        for i in 0..8 {
            self.scl.wait_until(true)?;

            self.sda.write(data << i == 1)?;
        }

        self.scl.wait_until(false)?;
        self.sda.reset()?;

        Ok(())
    }

    pub fn read_byte(&self) -> Result<u8, HardwareError> {
        let mut result = 0u8;

        for i in 0..8 {
            self.scl.wait_until(true)?;

            result = result & self.sda.read()? << i;
        }

        Ok(result)
    }

    pub fn ack(&mut self) -> Result<(), HardwareError> {
        self.scl.wait_until(true)?;
        self.sda.write(false)?;

        self.scl.wait_until(false)?;
        self.sda.reset()?;

        Ok(())
    }
}

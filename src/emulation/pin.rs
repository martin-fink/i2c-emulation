use sysfs_gpio::{Direction, Pin, Edge};
use std::error;
use sysfs_gpio;
use std::convert;
use std::fmt;
use std::error::Error;

#[derive(Debug)]
pub struct PinError {
    msg: String,
}

impl error::Error for PinError {
    fn description(&self) -> &str {
        &self.msg
    }
}

impl fmt::Display for PinError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.msg.fmt(f)
    }
}

impl convert::From<sysfs_gpio::Error> for PinError {
    fn from(error: sysfs_gpio::Error) -> Self {
        PinError {
            msg: String::from(error.description()),
        }
    }
}

pub struct I2CPin {
    hw_pin: Pin,
}

impl I2CPin {
    pub fn new(pin_number: u8) -> Self {
        let i2c_pin = I2CPin {
            hw_pin: Pin::new(pin_number as u64),
        };

        i2c_pin.hw_pin.set_direction(Direction::In).expect("Could not set direction on pin.");

        i2c_pin
    }

    pub fn read(&self) -> Result<u8, PinError> {
        self.hw_pin.set_direction(Direction::In)?;

        Ok(self.hw_pin.get_value()?)
    }

    pub fn write(&self, value: bool) -> Result<(), PinError> {
        if value {
            self.hw_pin.set_direction(Direction::In)?;
        } else {
            self.hw_pin.set_direction(Direction::Low)?;
        }

        Ok(())
    }

    pub fn reset(&self) -> Result<(), PinError> {
        self.write(true)
    }

    /// high: RisingEdge
    pub fn wait_until(&self, high: bool) -> Result<(), PinError> {
        self.hw_pin.set_edge(if high { Edge::RisingEdge } else { Edge::FallingEdge }).unwrap();
        let mut poller = self.hw_pin.get_poller()?;

        loop {
            match poller.poll(5000).expect("Unexpected error.") {
                Some(value) => return Ok(()),
                None => {}
            }
        }
    }
}


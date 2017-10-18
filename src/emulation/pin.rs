use sysfs_gpio::{Direction, Pin};
use std::error;
use sysfs_gpio;
use std::convert;
use std::fmt;

#[derive(Debug)]
pub struct PinError {
    cause: sysfs_gpio::Error,
}

impl error::Error for PinError {
    fn description(&self) -> &str {
        self.cause.description()
    }

    fn cause(&self) -> Option<&error::Error> {
        self.cause.cause()
    }
}

impl fmt::Display for PinError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.cause.fmt(f)
    }
}

impl convert::From<sysfs_gpio::Error> for PinError {
    fn from(error: sysfs_gpio::Error) -> Self {
        PinError {
            cause: error,
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

        i2c_pin.hw_pin.set_direction(Direction::In).expect("Could not set direction on pin. Does the pin exist?");

        i2c_pin
    }

    pub fn read(&self) -> Result<bool, PinError> {
        self.hw_pin.set_direction(Direction::In)?;

        Ok(self.hw_pin.get_value()? == 1)
    }

    pub fn write(&self, value: bool) -> Result<(), PinError> {
        if value {
            self.hw_pin.set_direction(Direction::In)?;
        } else {
            self.hw_pin.set_direction(Direction::Out)?;
            self.hw_pin.set_value(0)?;
        }

        Ok(())
    }
}


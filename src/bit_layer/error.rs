use std::{error, fmt, io, convert};
use sysfs_gpio;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    PinError(String),
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Io(ref e) => e.description(),
            Error::PinError(ref descr) => descr,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref e) => e.fmt(f),
            Error::PinError(ref descr) => write!(f, "PinError: {}", descr),
        }
    }
}

impl convert::From<sysfs_gpio::Error> for Error {
    fn from(prev: sysfs_gpio::Error) -> Self {
        Error::PinError(format!("PinError: {}", prev))
    }
}

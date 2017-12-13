use std::{error, fmt, io, convert};
use rppal::gpio;

#[derive(Debug)]
#[allow(dead_code)]
pub enum Error {
    Io(io::Error),
    Generic(String),
    UnexpectedSdaEdge,
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Io(ref e) => e.description(),
            Error::Generic(ref descr) => descr,
            Error::UnexpectedSdaEdge => "Unexpected sda edge",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref e) => e.fmt(f),
            Error::Generic(ref descr) => write!(f, "PinError: {}", descr),
            Error::UnexpectedSdaEdge => write!(f, "Unexpected sda edge"),
        }
    }
}

impl convert::From<gpio::Error> for Error {
    fn from(prev: gpio::Error) -> Self {
        Error::Generic(format!("PinError: {}", prev))
    }
}

use std::convert;
use std::fmt;

impl fmt::Display for RWBit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &RWBit::SlaveRead => f.write_str("SlaveRead"),
            &RWBit::SlaveWrite => f.write_str("SlaveWrite"),
        }
    }
}

pub enum RWBit {
    SlaveRead,
    SlaveWrite,
}

impl convert::From<u8> for RWBit {
    fn from(value: u8) -> Self {
        match value {
            0 => RWBit::SlaveWrite,
            1 => RWBit::SlaveRead,
            _ => {
                error!("Unexpected value {:b} for rw bit, assuming 1", value);
                RWBit::SlaveRead
            }
        }
    }
}

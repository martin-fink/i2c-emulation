use std::convert;

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

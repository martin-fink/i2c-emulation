use super::pin::I2CPin;
use super::register::I2CRegister;

enum ReadLoopState {
    Idle,
    WaitClockLow,
    WaitClockHigh,
}

enum I2CState {
    ReadI2CAddr,
    WriteRegister, // if repeated start -> ReadValue; else WriteValue
    WriteValue, // master writes; slave reads
    ReadValue, // slave writes; master reads
}

enum ReadWriteBit {
    Read,
    Write,
}

impl ReadWriteBit {
    fn from_value(value: u8) -> Self {
        if value & 1 == 1 {
            ReadWriteBit::Read
        } else {
            ReadWriteBit::Write
        }
    }
}

const PIN_SCL: u8 = 24; // GPIO 5
const PIN_SDA: u8 = 25; // GPIO 6

const STANDARD_ADDRESS: u8 = 0b0101_0101; // dec: 85

/// rw_bit: low: write; high: read
/// address: i²c address, only 7bit mode supported
pub struct HWThread {
    address: u8,
    rw_bit: ReadWriteBit,
    registers: Vec<I2CRegister>,
    current_register: Option<u8>,
    scl: I2CPin,
    sda: I2CPin,
}

impl HWThread {
    pub fn validate_address_7b(address: u8) -> bool {
        if address > (1 << 7) {
            return false;
        }

        match address {
            0b0000_0000
            | 0b0000_0001
            | 0b0000_0010
            | 0b0000_0011
            | 0b0000_0100 ... 0b0000_1000
            | 0b0111_1100 ... 0b1000_0000
            | 0b0111_1000 ... 0b0111_1100 => false,
            _ => true,
        }
    }

    pub fn new() -> Self {
        assert!(HWThread::validate_address_7b(STANDARD_ADDRESS), "Invalid I²C address: {}", STANDARD_ADDRESS);

        HWThread {
            address: STANDARD_ADDRESS,
            rw_bit: ReadWriteBit::Read,
            registers: Vec::new(),
            current_register: Option::None,
            scl: I2CPin::new(PIN_SCL),
            sda: I2CPin::new(PIN_SDA),
        }
    }

    pub fn start(mut self) {
        let mut state = I2CState::ReadI2CAddr;
        loop {
            match state {
                I2CState::ReadI2CAddr => {
                    self.current_register = Option::None;
                    self.wait_for_start();
                    if let Ok(value) = self.read_byte() {
                        if (value >> 1) == self.address {
                            self.ack();
                            self.rw_bit = ReadWriteBit::from_value(value);
                        }

                        // TODO: change state to next step
                    } else {
                        panic!("Could not read byte.");
                    }
                },
                I2CState::WriteRegister => {},
                I2CState::WriteValue => {},
                I2CState::ReadValue => {},
            }
        }

        // TODO: cleanup pins
    }

    fn ack(&self) {
        unimplemented!()
    }

    fn wait_for_start(&self) -> Result<(), ()> {
        unimplemented!()
    }

    fn read_byte(&self) -> Result<u8, ()> {
        unimplemented!()
    }

    fn write_byte(&self, value: u8) -> Result<(), ()> {
        unimplemented!()
    }
}

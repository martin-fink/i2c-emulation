use super::pin::I2CPin;
use super::register::I2CRegister;

#[derive(Copy, Clone, Debug)]
enum ReadLoopState {
    Idle,
    WaitClockLow,
    WaitClockHigh,
}

#[derive(Copy, Clone, Debug)]
enum I2CState {
    ReadI2CAddr,
    // if repeated start -> ReadValue; else WriteValue
    SlaveReadValue,
    // master writes; slave reads
    SlaveWriteValue,
    // slave writes; master reads
}

#[derive(Copy, Clone, Debug)]
enum ReadWriteBit {
    SlaveRead,
    SlaveWrite,
}

#[derive(Copy, Clone, Debug)]
enum ReadByteResult {
    Start,
    Stop,
    Byte(u8),
}

impl ReadWriteBit {
    fn from_value(value: u8) -> Self {
        if value & 1 == 1 {
            ReadWriteBit::SlaveRead
        } else {
            ReadWriteBit::SlaveWrite
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
                    self.wait_for_start().expect("Expected start.");

                    let result = self.read().expect("Could not read byte.");

                    if let ReadByteResult::Byte(value) = result {
                        if value >> 1 == self.address {
                            self.ack();

                            state = match ReadWriteBit::from_value(value) {
                                ReadWriteBit::SlaveWrite => {
                                    I2CState::SlaveWriteValue
                                }
                                ReadWriteBit::SlaveRead => {
                                    self.current_register = None;
                                    I2CState::SlaveReadValue
                                }
                            }
                        }
                    }
                }
                I2CState::SlaveReadValue => {
                    match self.read().expect("Could not read.") {
                        ReadByteResult::Byte(value) => {
                            if let Some(register) = self.current_register {
                                let register = register as usize;
                                if register >= self.registers.len() {
                                    panic!("Register {} does not exist.", register)
                                }

                                self.registers[register].set_value(value);
                            } else {
                                self.current_register = Some(value);
                            }
                            self.ack();
                        }
                        ReadByteResult::Stop => state = I2CState::ReadI2CAddr,
                        ReadByteResult::Start => state = I2CState::ReadI2CAddr,
                    }
                }
                I2CState::SlaveWriteValue => {
                    if let ReadByteResult::Byte(value) = self.read().expect("Could not read.") {
                        if value >> 1 == self.address && value & 1 == 0 {
                            self.write_current_register();

                            state = I2CState::ReadI2CAddr;
                        }
                    }
                }
            }
        }

        // TODO: cleanup pins
    }

    fn write_current_register(&self) -> Result<(), ()> {
        if let Some(register) = self.current_register {
            let register = register as usize;
            if register >= self.registers.len() {
                return Err(());
            }

            return self.write(self.registers[register].get_value());
        }

        Err(())
    }

    fn set_current_register(&mut self, value: u8) -> Result<(), ()> {
        if let Some(register) = self.current_register {
            let register = register as usize;
            if register >= self.registers.len() {
                return Err(());
            }

            self.registers[register].set_value(value);
            return Ok(());
        }

        Err(())
    }

    fn ack(&self) {
        unimplemented!()
    }

    fn read(&self) -> Result<ReadByteResult, ()> {
        unimplemented!()
    }

    fn write(&self, byte: u8) -> Result<(), ()> {
        unimplemented!()
    }

    fn wait_for_start(&self) -> Result<(), ()> {
        let result = self.read().expect("Could not read byte");
        match result {
            ReadByteResult::Start => Ok(()),
            _ => Err(()),
        }
    }
}

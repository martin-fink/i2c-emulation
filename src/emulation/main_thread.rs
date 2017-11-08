use super::pin::I2CPin;
use super::register::I2CRegister;
use super::bit_layer::BitLayer;

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

const PIN_SCL: u8 = 5; // GPIO 5
const PIN_SDA: u8 = 6; // GPIO 6

const STANDARD_ADDRESS: u8 = 0b0101_0101; // dec: 85

/// rw_bit: low: write; high: read
/// address: i²c address, only 7bit mode supported
pub struct MainThread {
    address: u8,
    registers: Vec<I2CRegister>,
    current_register: Option<usize>,
    hw_layer: BitLayer,
}

impl MainThread {
    pub fn validate_address_7b(address: u8) -> bool {
        if address > (1 << 7) {
            return false;
        }

        match address {
            0b_0000_0000
            | 0b_0000_0001
            | 0b_0000_0010
            | 0b_0000_0011
            | 0b_0000_0100 ... 0b_0000_1000
            | 0b_0111_1100 ... 0b_1000_0000
            | 0b_0111_1000 ... 0b_0111_1100 => false,
            _ => true,
        }
    }

    pub fn new() -> Self {
        assert!(MainThread::validate_address_7b(STANDARD_ADDRESS), "Invalid I²C address: {}", STANDARD_ADDRESS);

        MainThread {
            address: STANDARD_ADDRESS,
            registers: Vec::new(),
            current_register: Option::None,
            hw_layer: BitLayer::new(PIN_SDA, PIN_SCL),
        }
    }

    pub fn start(mut self) {
        let mut state = I2CState::ReadI2CAddr;
        loop {
            match state {
                I2CState::ReadI2CAddr => {
                    self.hw_layer.wait_for_start().expect("Expected start.");

                    let value = self.hw_layer.read_byte().expect("Could not read byte.");

                    if value >> 1 == self.address {
                        self.hw_layer.ack();

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
                I2CState::SlaveReadValue => {
                    let value = self.hw_layer.read_byte().expect("Could not read.");
                    if let Some(register) = self.current_register {
                        if register >= self.registers.len() {
                            panic!("Register {} does not exist.", register)
                        }

                        self.registers[register].set_value(value);
                    } else {
                        self.current_register = Some(value as usize);
                    }
                    self.hw_layer.ack();
                }
                I2CState::SlaveWriteValue => {
                    let value = self.hw_layer.read_byte().expect("Could not read.");
                    if value >> 1 == self.address && value & 1 == 0 {
                        self.get_current_register();

                        state = I2CState::ReadI2CAddr;
                    }
                }
            }
        }

        // TODO: cleanup pins
    }

    fn get_current_register(&self) -> Result<u8, String> {
        if let Some(register) = self.current_register {
            if register >= self.registers.len() {
                return Err(format!("Register {:x} not found.", register));
            }

            Ok(self.registers[register].get_value())
        } else {
            Err(String::from("Current register not set."))
        }
    }

    fn set_current_register(&mut self, value: u8) -> Result<(), String> {
        if let Some(register) = self.current_register {
            if register >= self.registers.len() {
                return Err(format!("Register {:x} not found.", register));
            }

            self.registers[register].set_value(value);
            return Ok(());
        }

        Err(String::from("Current register not set."))
    }
}

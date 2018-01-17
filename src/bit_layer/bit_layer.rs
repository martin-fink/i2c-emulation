use super::I2CProtocol;
use super::Error;
use super::RWBit;
use super::pin_thread;
use super::PinThread;
use super::pin_thread::{PinType, Message};
use std::sync::mpsc;
use std::thread;
use rppal::gpio::{Gpio, Level, Mode};

enum RepeatedStartData {
    Data(u8),
    RepeatedStart(u8),
}

struct Pin {
    pin_number: u8,
    gpio: Gpio,
}

impl Pin {
    fn new(pin_number: u8) -> Self {
        Pin {
            pin_number,
            gpio: Gpio::new().unwrap(),
        }
    }

    fn reset(&mut self) {
        self.gpio.set_mode(self.pin_number, Mode::Input)
    }

    fn get_value(&mut self) -> Result<u8, Error> {
        Ok(self.gpio.read(self.pin_number)? as u8)
    }

    fn set_logiclvl(&mut self, value: Level) {
        self.gpio.write(self.pin_number, value)
    }

    fn set_write_mode(&mut self) {
        self.gpio.set_mode(self.pin_number, Mode::Output);
        self.gpio.write(self.pin_number, Level::High);
    }
}

pub struct BitLayer<P> where P: I2CProtocol {
    implementation: P,
    sda: Pin,
    scl: Pin,
    current_register: Option<usize>,
    sda_num: u8,
    scl_num: u8,
    rx: mpsc::Receiver<Message>,
    tx: mpsc::SyncSender<Message>
}

trait I2CPin {
    fn set_write_mode(&self) -> Result<(), Error>;

    fn reset(&self) -> Result<(), Error>;
}

impl<P> BitLayer<P> where P: I2CProtocol {
    pub fn new(implementation: P, sda_num: u8, scl_num: u8) -> Self {
        let (tx, rx) = mpsc::sync_channel::<pin_thread::Message>(0);

        BitLayer {
            implementation,
            sda: Pin::new(sda_num),
            scl: Pin::new(scl_num),
            current_register: None,
            sda_num,
            scl_num,
            rx,
            tx,
        }
    }

    pub fn run(mut self) -> Result<(), Error> {
        trace!("Start BitLayer Thread");

        let scl_num = self.scl_num;
        let tx = self.tx.clone();
        thread::spawn(move || {
            info!("Spawned scl thread");
            PinThread::new(scl_num, tx, pin_thread::PinType::Scl)
                .run();
        });

        let sda_num = self.sda_num;
        let tx = self.tx.clone();
        thread::spawn(move || {
            info!("Spawned sda thread");
            PinThread::new(sda_num, tx, pin_thread::PinType::Sda)
                .run();
        });

        loop {
            let read_message = self.rx.recv().expect("Channel was closed.");

            match read_message.pin_type {
                PinType::Sda => {
                    if read_message.value == 0 && self.scl.get_value()? == 1 {
                        let (address, rw) = self.read_address_and_rw()?;

                        if self.implementation.check_address(address) {
                            self.ack()?;

                            let result = self.read_data();

                            if result.is_err() {
                                info!("Address 0x{:x} matched!", address);
                                info!("RW: {}", rw);
                                continue;
                            }

                            let result = result.unwrap();

                            self.current_register = Some(result as usize);

                            self.ack()?;

                            let result = self.read_data_or_repeated_start().unwrap();

                            match result {
                                RepeatedStartData::Data(value) => {
                                    self.ack()?;

                                    let register_address = self.current_register.unwrap();
                                    self.implementation.set_register(register_address, value);
                                }
                                RepeatedStartData::RepeatedStart(address) => {
                                    if !self.implementation.check_address(address) {
                                        continue;
                                    }

                                    let current_register = self.current_register.unwrap();
                                    let byte = self.implementation.get_register(current_register).unwrap();

                                    // cheaty ack hack
                                    self.sda.set_write_mode();

                                    self.sda.set_logiclvl(Level::Low);

                                    loop {
                                        let read_result = self.rx.recv().unwrap();

                                        if let PinType::Scl = read_result.pin_type {
                                            if read_result.value == 1 {
                                                self.sda.reset();
                                                break;
                                            }
                                        }
                                    }

                                    self.write_byte(byte)?;
                                }
                            }
                        } else {
                            trace!("Address 0x{:08x} did not match", address);
                        }
                    }
                }
                PinType::Scl => {}
            }
        }
    }

    fn read_data(&mut self) -> Result<u8, Error> {
        trace!("Reading data");

        let mut sda_current_value = self.sda.get_value()?;
        let mut value = 0;
        let mut bytes_read = 0u32;

        while bytes_read < 8u32 {
            let read_result = self.rx.recv().unwrap();

            match read_result.pin_type {
                PinType::Scl => {
                    if read_result.value == 1 {
                        value = (value << 1) | sda_current_value;
                        bytes_read += 1;
                    }
                }
                PinType::Sda => {
                    if self.scl.get_value()? == 1 {
                        return Err(Error::UnexpectedSdaEdge);
                    }
                    sda_current_value = read_result.value
                }
            }
        }

        Ok(value)
    }

    fn read_data_or_repeated_start(&mut self) -> Result<RepeatedStartData, Error> {
        let mut repeated_start = false;
        let mut value = 0;
        let mut bits_read = 0u32;

        while bits_read < 8u32 {
            let read_result = self.rx.recv().unwrap();

            match read_result.pin_type {
                PinType::Scl => if read_result.value == 1 {
                    value = (value << 1) | self.sda.get_value()?;
                    bits_read += 1;
                }
                PinType::Sda => if read_result.value == 0 && self.scl.get_value()? == 1 {
                    repeated_start = true;
                    bits_read = 0;
                    value = 0;
                }
            }
        }

        Ok(if repeated_start {
            let (address, _) = self.split_address_and_rw(value);
            RepeatedStartData::RepeatedStart(address)
        } else {
            RepeatedStartData::Data(value)
        })
    }

    fn read_address_and_rw(&mut self) -> Result<(u8, RWBit), Error> {
        let mut sda_current_value = self.sda.get_value()?;
        let mut value = 0;
        let mut bytes_read = 0u32;

        while bytes_read < 8u32 {
            let read_result = self.rx.recv().unwrap();

            match read_result.pin_type {
                PinType::Scl => {
                    if read_result.value == 1 {
                        value = (value << 1) | sda_current_value;
                        bytes_read += 1;
                    }
                }
                PinType::Sda => sda_current_value = read_result.value
            }
        }

        Ok(self.split_address_and_rw(value))
    }

    fn write_byte(&mut self, byte: u8) -> Result<(), Error> {
        self.sda.set_write_mode();

        let mut index = 7;

        while index >= 0 {
            let read_message = self.rx.recv().unwrap();

            match read_message.pin_type {
                PinType::Scl => {
                    if read_message.value == 0 {
//                        if index == 7 {
//                            self.sda.reset();
//                        }
                        self.sda.set_logiclvl(if byte & (1 << index) == 0 {
                            Level::Low
                        } else {
                            Level::High
                        });

                        index -= 1;
                    }
                }
                _ => {}
            }
        }

        self.sda.reset();

        Ok(())
    }

    fn ack(&mut self) -> Result<(), Error> {
        let mut ack_sent = false;

        self.sda.set_write_mode();

        self.sda.set_logiclvl(Level::Low);

        // wait until scl is low and the slave is allowed to change sda
        //        loop {
        //            let read_result = self.rx.recv().unwrap();
        //
        //            if let PinType::Scl = read_result.pin_type {
        //                if read_result.value == 0 { break }
        //            }
        //        }

        loop {
            let read_result = self.rx.recv().unwrap();

            if let PinType::Scl = read_result.pin_type {
                if read_result.value == 0 {
                    if ack_sent { break } else { ack_sent = true }
                }
            }
        }

        self.sda.reset();

        Ok(())
    }

    fn split_address_and_rw(&self, address_and_rw: u8) -> (u8, RWBit) {
        (address_and_rw >> 1, RWBit::from(address_and_rw & 0x1))
    }
}


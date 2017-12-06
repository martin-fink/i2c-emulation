use super::I2CProtocol;
use super::Error;
use super::RWBit;
use super::pin_thread;
use super::PinThread;
use super::pin_thread::{PinType, Message};
use std::sync::mpsc;
use std::thread;
use rppal::gpio::{Gpio, Level, Mode};

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
//                        info!("Received start");
                        let (address, rw) = self.read_address_and_rw()?;

                        if self.implementation.check_address(address) {
                            println!("Address {:07b} matched!", address);
                            println!("RW: {}", rw);
                            self.ack()?;
                            // TODO: do the rest of the implementation
                        } else {
                            trace!("Address {} did not match", address);
                        }
                    }
                }
                PinType::Scl => {}
            }
        }
    }

    fn read_address_and_rw(&mut self) -> Result<(u8, RWBit), Error> {
//        trace!("Reading address and rw");
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

    #[allow(dead_code)]
    fn write_byte(&mut self, byte: u8) -> Result<(), Error> {
        trace!("Writing byte");

        self.sda.set_write_mode();

        let mut index = 0;
        while index < 8 {
            let read_message = self.rx.recv().unwrap();

            if let PinType::Scl = read_message.pin_type {
                // change sda when scl is low
                if read_message.value == 0 {
                    trace!("scl is low, setting sda to {}", read_message.value << index);
                    self.sda.set_logiclvl(if byte << index == 0 {
                        Level::Low
                    } else {
                        Level::High
                    });
                    index += 1;
                } else {
                    trace!("scl is high");
                }
            }
        }

        self.sda.reset();

        Ok(())
    }

    fn ack(&mut self) -> Result<(), Error> {
        let mut ack_sent = false;

        self.sda.set_write_mode();

        self.sda.set_logiclvl(Level::Low);

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


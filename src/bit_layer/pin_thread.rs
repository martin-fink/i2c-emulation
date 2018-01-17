use std::sync::mpsc::SyncSender;
use std::fmt;
use rppal::gpio::{Gpio, Mode};

impl fmt::Display for PinType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match *self {
            PinType::Sda => "sda",
            PinType::Scl => "scl",
        })
    }
}

#[derive(Copy, Clone)]
pub enum PinType {
    Sda,
    Scl,
}

pub struct Message {
    pub pin_type: PinType,
    pub value: u8,
}

pub struct PinThread {
    gpio: Gpio,
    pin_number: u8,
    sender: SyncSender<Message>,
    pin_type: PinType,
}

impl PinThread {
    pub fn new(pin_number: u8, sender: SyncSender<Message>, pin_type: PinType) -> Self {
        PinThread {
            gpio: Gpio::new().unwrap(),
            pin_number,
            sender,
            pin_type,
        }
    }

    pub fn run(mut self) {
        trace!("Run method started...");

        self.gpio.set_mode(self.pin_number, Mode::Input);
        let mut last_value = self.gpio.read(self.pin_number).expect("could not read") as u8;

        loop {
            let value = self.gpio.read(self.pin_number).expect("Could not read") as u8;

            if value != last_value {
                last_value = value;

                self.sender
                    .send(Message {
                        pin_type: self.pin_type,
                        value,
                    })
                    .expect("Could not send.");
            }
        }
    }
}

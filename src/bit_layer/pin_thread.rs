use sysfs_gpio::{Pin, Edge, Direction};
use std::sync::mpsc::SyncSender;
use std::fmt;

impl fmt::Display for PinType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            &PinType::Sda => "sda",
            &PinType::Scl => "scl",
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
    pin: Pin,
    sender: SyncSender<Message>,
    pin_type: PinType,
}

impl PinThread {
    pub fn new(pin_number: u8, sender: SyncSender<Message>, pin_type: PinType) -> Self {
        PinThread {
            pin: Pin::new(pin_number as u64),
            sender,
            pin_type,
        }
    }

    pub fn run(self) {
        trace!("Run method started...");

        self.pin.set_direction(Direction::In).unwrap();
        self.pin.set_edge(Edge::BothEdges).unwrap();
        let mut poller = self.pin.get_poller().expect("Could not get poller.");

        loop {
            match poller.poll(5000).expect("Error reading from pin.") {
                Some(value) => {
                    info!("Read {} at pin {}", value, self.pin_type);
                    self.sender.send(Message {
                        pin_type: self.pin_type,
                        value,
                    }).unwrap();
                }
                None => {
                    info!("Poller at pin {} timed out, retrying...", self.pin_type);
                }
            }
        }
    }
}

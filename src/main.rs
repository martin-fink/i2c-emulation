#[macro_use]
extern crate log;
extern crate env_logger;
extern crate sysfs_gpio;
extern crate tokio_core;
extern crate futures;
extern crate rppal;

mod bit_layer;

use bit_layer::{I2CProtocol, BitLayer};

struct ProtocolImplementation {
    address: u8,
    registers: Vec<u8>,
}

impl ProtocolImplementation {
    fn new(address: u8, registers: Vec<u8>) -> Self {
        ProtocolImplementation {
            address,
            registers,
        }
    }
}

impl I2CProtocol for ProtocolImplementation {
    fn check_address(&self, address: u8) -> bool {
        self.address == address
    }

    fn set_register(&mut self, register: usize, data: u8) -> bool {
        if self.registers.len() <= register {
            return false;
        }

        self.registers[register] = data;
        true
    }

    fn get_register(&self, register: usize) -> Option<u8> {
        if self.registers.len() <= register {
            None
        } else {
            Some(self.registers[register])
        }
    }
}

fn main() {
    env_logger::init().expect("Could not init logger.");

    trace!("Setting up main");

    let protocol = ProtocolImplementation::new(0b111, vec![0, 0, 0]);
    let bit_layer = BitLayer::new(protocol, 6, 5);

    match bit_layer.run() {
        Ok(()) => {}
        Err(error) => eprintln!("Error: {}", error),
    }
}

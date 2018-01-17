extern crate chrono;
extern crate clap;
extern crate fern;
extern crate futures;
#[macro_use]
extern crate log;
extern crate rppal;
extern crate sysfs_gpio;
extern crate tokio_core;

mod bit_layer;

use bit_layer::{BitLayer, I2CProtocol};
use clap::{App, Arg};

struct ProtocolImplementation {
    address: u8,
    registers: Vec<u8>,
}

impl ProtocolImplementation {
    fn new(address: u8, registers: Vec<u8>) -> Self {
        ProtocolImplementation { address, registers }
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
    let matches = App::new("i2c_emulation")
        .version("0.1")
        .author("Martin Fink <martinfink99@gmail.com")
        .about("Emulates a i2c slave")
        .arg(
            Arg::with_name("address")
                .value_name("ADDRESS")
                .required(true)
                .help("Sets the slave address")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Set the level of verbosity"),
        )
        .get_matches();

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .chain(std::io::stderr())
        .chain(fern::log_file("output.log").expect("Could not open log file"))
        .level(match matches.occurrences_of("v") {
            0 => log::LogLevelFilter::Error,
            1 => log::LogLevelFilter::Warn,
            2 => log::LogLevelFilter::Info,
            3 => log::LogLevelFilter::Debug,
            _ => log::LogLevelFilter::Trace,
        })
        .apply()
        .expect("Could not init logger");

    let address = matches.value_of("address").unwrap();
    let address = address
        .parse::<u8>()
        .expect("Address must be a valid integer");

    info!("Using slave address 0x{:x}", address);

    let protocol = ProtocolImplementation::new(address, vec![0b1011_1001, 0xAA, 0b0111_0000]);
    let bit_layer = BitLayer::new(protocol, 6, 5);

    match bit_layer.run() {
        Ok(()) => error!("Returned from loop"),
        Err(error) => error!("Error: {}", error),
    }
}

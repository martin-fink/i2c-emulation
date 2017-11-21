use sysfs_gpio::{Pin, Edge, Direction};
use super::I2CProtocol;
use super::Error;
use super::RWBit;
use super::pin_thread;
use super::PinThread;
use super::pin_thread::{PinType, Message};
use std::sync::mpsc;
use std::thread;

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


impl<P> BitLayer<P> where P: I2CProtocol {
    pub fn new(implementation: P, sda_num: u8, scl_num: u8) -> Self {
        let (tx, rx) = mpsc::sync_channel::<pin_thread::Message>(0);

        BitLayer {
            implementation,
            sda: Pin::new(sda_num as u64),
            scl: Pin::new(scl_num as u64),
            current_register: None,
            sda_num,
            scl_num,
            rx,
            tx,
        }
    }

    pub fn run(self) -> Result<(), Error> {
        trace!("Start BitLayer Thread");

        let scl_num = self.scl_num;
        let tx = self.tx.clone();
        thread::spawn(move|| {
            info!("Spawned scl thread");
            PinThread::new(scl_num, tx, pin_thread::PinType::Scl)
                .run();
        });

        let sda_num = self.sda_num;
        let tx = self.tx.clone();
        thread::spawn(move|| {
            info!("Spawned sda thread");
            PinThread::new(sda_num, tx, pin_thread::PinType::Sda)
                .run();
        });

        loop {
            let read_message = self.rx.recv().expect("Channel was closed.");

            match read_message.pin_type {
                PinType::Sda => {
                    if read_message.value == 1 {
                        info!("Received start");
                        let (address, rw) = self.read_address_and_rw()?;

                        if self.implementation.check_address(address) {
                            println!("Address matched!");
                            // TODO: do the rest of the imeplementation
                        } else {
                            trace!("Address did not match");
                        }
                    }
                }
                PinType::Scl => {}
            }
        }
    }

    fn read_address_and_rw(&self) -> Result<(u8, RWBit), Error> {
        trace!("Reading address and rw");
        let mut sda_current_value = 0;
        let mut value = 0;
        let mut bytes_read = 0u32;

        while bytes_read < 8u32 {
            let read_result = self.rx.recv().unwrap();

            match read_result.pin_type {
                PinType::Scl => {
                    if read_result.value == 1 {
                        value = (value << 1) & (read_result.value & 0x1);
                    }
                    bytes_read += 1;
                }, PinType::Sda => sda_current_value = read_result.value
            }
        }

        Ok(self.split_address_and_rw(value))
    }

    fn write_byte(&self, byte: u8) -> Result<(), Error> {
        trace!("Writing byte");

        info!("Setting sda to no interrupt mode");
        self.sda.set_edge(Edge::NoInterrupt)?;

        info!("Setting sda to output mode");
        self.sda.set_direction(Direction::Out)?;

        let mut index = 0;
        while index < 8 {
            let read_message = self.rx.recv().unwrap();

            if let PinType::Scl = read_message.pin_type {
                // change sda when scl is low
                if read_message.value == 0 {
                    trace!("scl is low, setting sda to {}", read_message.value << index);
                    self.sda.set_direction(if read_message.value << index == 0 {
                        Direction::Low
                    } else {
                        Direction::High
                    })?;
                    index += 1;
                } else {
                    trace!("scl is high");
                }
            }
        }

        info!("Resetting interrupt and direction for sda");
        self.sda.set_direction(Direction::In)?;
        self.sda.set_edge(Edge::BothEdges)?;

        Ok(())
    }

    fn ack(&self) -> Result<(), Error> {
        trace!("Sending ack");
        // TODO: implement ack

        Ok(())
    }

    fn split_address_and_rw(&self, address_and_rw: u8) -> (u8, RWBit) {
        (address_and_rw >> 1, RWBit::from(address_and_rw & 0x1))
    }

    /*
        fn read_byte(&self) -> Result<u8, Error> {
            trace!("Reading byte");
            let mut byte = 0u8;

            for i in 0..8 {
                self.wait_for(&self.scl, Edge::RisingEdge)?;
                let value = self.sda.get_value()?;

                byte = value << i;
            }

            info!("Read byte {}", byte);
            Ok(byte)
        }

        fn wait_for_start(&self) -> Result<(), Error> {
            trace!("Waiting for start");
            self.wait_for(&self.sda, Edge::FallingEdge)?;
            self.assert_input(&self.scl, 0)
        }

        fn assert_input(&self, pin: &Pin, expected: u8) -> Result<(), Error> {
            assert!(expected == 0 || expected == 1, "Expected input must be 1 or 0");

            let actual = pin.get_value()?;

            if actual == expected {
                Ok(())
            } else {
                Err(Error::AssertionError(format!("expected input {} at pin {} does not match actual input {}",
                                                  expected, pin.get_pin(), actual)))
            }
        }

        fn wait_for(&self, pin: &Pin, edge: Edge) -> Result<(), Error> {
            trace!("Setting edge {:?} for pin {}", edge, pin.get_pin());
            pin.set_edge(edge)?;
            let mut poller = pin.get_poller()?;

            loop {
                trace!("Polling for edge {:?} at pin {} for at least {} ms",
                       edge, pin.get_pin(), DEFAULT_POLL_TIMEOUT);
                match poller.poll(DEFAULT_POLL_TIMEOUT)? {
                    Some(value) => return Ok(()),
                    None => info!("Exceeded timeout of {} ms for pin {} and edge {:?}",
                                  DEFAULT_POLL_TIMEOUT, pin.get_pin(), edge)
                }
            }
        }
    */
}


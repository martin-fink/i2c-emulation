#[macro_use]
extern crate log;
extern crate env_logger;
extern crate sysfs_gpio;
extern crate tokio_core;
extern crate futures;

mod bit_layer;
mod protocol;

fn main() {
    env_logger::init().expect("Could not init logger.");
    trace!("Setting up main");
}

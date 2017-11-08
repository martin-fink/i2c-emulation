extern crate sysfs_gpio;

mod emulation;

use emulation::MainThread;

fn main() {
    println!("Hello, world!");

    let mut main_thread = MainThread::new();

    main_thread.start();
}

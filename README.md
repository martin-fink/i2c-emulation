I²C Emulation
=============

## What is this for?
This project tries to emulate a I²C slave by bitbanging the I²C
protocol.

## Limitations
The main limitation is speed. It takes a lot of CPU cycles to read/write
from the GPIO pins. To use this program, you should set your I²C master to the
lowest speed possible, maybe even underclocking it.

## Build
To build this executable for a Raspberry Pi, you need to install
rust for the `arm-unknown-linux-gnueabihf` toolchain.

Rustup can be used to install the compiler, docs and stdlib for
the desired architecture. Rustup can be installed with the following command:

```bash
curl https://sh.rustup.rs -sSf | sh
```

To add the target, run the following command:

```bash
rustup target add arm-unknown-linux-gnueabihf
```

For a build, you need to build the executable with the following command:

```bash
cargo build --target=arm-unknown-linux-gnueabihf
```

After building, you can use scp or rsync to copy the binary to the Pi.

## Credits

- Martin Fink
- Tobias Oberdörfer

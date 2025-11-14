# An Operation System on Raspberry PI wrote in Rust

The project is an OS implementation on Raspberry PI SBC, which was wrote in Rut.

The code was derived from [Operating System development tutorials in Rust on the Raspberry Pi](https://github.com/rust-embedded/rust-raspberrypi-OS-tutorials).

## Run on target

### U-boot

Run with u-boot:

```
U-Boot> fatload mmc 0 0x80000 rpi_os.img
U-Boot> go 0x80000
```

## Current Status:

- Start code
- Early drivers: GPIO & Uart
- Timer
- Test on QEMU
- Test on RPI4b board

## Useful utils

- [Rust OSDev](https://github.com/rust-osdev)

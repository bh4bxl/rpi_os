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

## Debug with QEMU

### Build with debug symbol

```
cargo build
```

### Create kernel image

```
rust-objcopy -O binary target/aarch64-unknown-none/debug/kernel rpi_os.img
```

### Start QEMU with debug

```
qemu-system-aarch64 -M raspi4b -serial stdio -display none -kernel rpi_os.img -S -gdb tcp::1234
```

### Connect to the QEMU with gdb

```
rust-gdb
...
(gdb) add-symbol-file target/aarch64-unknown-none/debug/kernel
...
(gdb) target remote localhost:1234
...
(gdb) b main.rs:25
...
(gdb) c
...
```

## Current Status:

- Start code
- Early drivers: GPIO & Uart
- Timer
- Test on QEMU
- Test on RPI4b board

## Useful utils

- [Rust OSDev](https://github.com/rust-osdev)

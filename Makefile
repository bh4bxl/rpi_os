# build configurations
TARGET = aarch64-unknown-none-softfloat
KERNEL_BIN = kernel8.img
QEMU_CMD = qemu-system-aarch64
QEMU_MACHINE_TYPE = raspi4b
QEMU_ARGS = -serial stdio -display none
LD_SCRIPT_PATH = $(shell pwd)/src/boards/rpi4/
KERNEL_MEMORY_LAYOUT = memory.x
RUSTC_MISC_ARGS = -C target-cpu=cortex-a72
KERNEL_ELF = target/$(TARGET)/release/kernel
KERNEL_ELF_DEPS = $(filter-out %: ,$(file < $(KERNEL_ELF).d))
RELEASE = --release

# Build commands

FEATURES = #--features bsp_rpi4
COMPILER_ARGS = --target=$(TARGET) \
  $(FEATURES) \
  $(RELEASE)

RUSTFLAGS = $(RUSTC_MISC_ARGS) \
  -C link-arg=--library-path=$(LD_SCRIPT_PATH) \
  -C link-arg=--script=$(KERNEL_MEMORY_LAYOUT)

RUSTFLAGS_PEDANTIC = $(RUSTFLAGS) \
  #-D warnings  \
  #-D missing_docs

RUSTC_CMD = cargo rustc $(COMPILER_ARGS)
OBJCOPY_CMD = rust-objcopy \
  --strip-all \
  -O binary

## Targets
.PHONY: all qemu clean

all: $(KERNEL_BIN)

## Build and generate kernel binary

$(KERNEL_BIN): $(KERNEL_ELF)
	@$(OBJCOPY_CMD) $(KERNEL_ELF) $(KERNEL_BIN)

$(KERNEL_ELF): $(KERNEL_ELF_DEPS)
	RUSTFLAGS="$(RUSTFLAGS_PEDANTIC)" $(RUSTC_CMD)

qemu: $(KERNEL_BIN)
	$(QEMU_CMD) -M $(QEMU_MACHINE_TYPE) $(QEMU_ARGS) -kernel $(KERNEL_BIN)

## Clean
clean:
	rm -rf target $(KERNEL_BIN)

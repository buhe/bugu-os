# Building
TARGET := riscv64gc-unknown-none-elf
MODE := release
KERNEL_ELF := target/$(TARGET)/$(MODE)/os
KERNEL_BIN := $(KERNEL_ELF).bin

# BOARD
BOARD ?= k210
SBI ?= rustsbi
BOOTLOADER := ../bootloader/$(SBI)-$(BOARD).bin
K210_BOOTLOADER_SIZE := 131072
LOG ?= TRACE

# Run K210
K210-SERIALPORT	= /dev/tty.usbserial-35525425B50
K210-BURNER = ../tools/kflash.py

# Binutils
OBJCOPY := rust-objcopy --binary-architecture=riscv64

build: env $(KERNEL_BIN)

env:
	(rustup target list | grep "riscv64gc-unknown-none-elf (installed)") || rustup target add $(TARGET)
	cargo install cargo-binutils 
	rustup component add rust-src
	rustup component add llvm-tools-preview

$(KERNEL_BIN): kernel
	@$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary $@

kernel:
	@echo Platform: $(BOARD)
	@cd ../user && make build
	@cargo clean
	@cargo build --release

clean:
	@cargo clean

test: env
	@echo Platform: $(BOARD)
	@cargo clean
	@cd ../user && make build
	@cargo test --no-run
	$(eval override KERNEL_ELF := $(shell cargo test --no-run --message-format=json | jq -r "select(.profile.test == true) | .filenames[]"))
	@$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary $(KERNEL_BIN)
	(which $(K210-BURNER)) || (cd .. && git clone https://github.com/sipeed/kflash.py.git && mv kflash.py tools)
	@cp $(BOOTLOADER) $(BOOTLOADER).copy
	@dd if=$(KERNEL_BIN) of=$(BOOTLOADER).copy bs=$(K210_BOOTLOADER_SIZE) seek=1
	@mv $(BOOTLOADER).copy $(KERNEL_BIN)
	@sudo chmod 777 $(K210-SERIALPORT)
	python3 $(K210-BURNER) -p $(K210-SERIALPORT) -b 1500000 $(KERNEL_BIN)
	python3 -m serial.tools.miniterm --eol LF --dtr 0 --rts 0 --filter direct $(K210-SERIALPORT) 115200


doc:
	@git add -A
	@git commit -a -m"update docs"
	@git push

run: run-inner

run-inner: build
	(which $(K210-BURNER)) || (cd .. && git clone https://github.com/sipeed/kflash.py.git && mv kflash.py tools)
	@cp $(BOOTLOADER) $(BOOTLOADER).copy
	@dd if=$(KERNEL_BIN) of=$(BOOTLOADER).copy bs=$(K210_BOOTLOADER_SIZE) seek=1
	@mv $(BOOTLOADER).copy $(KERNEL_BIN)
	@sudo chmod 777 $(K210-SERIALPORT)
	python3 $(K210-BURNER) -p $(K210-SERIALPORT) -b 1500000 $(KERNEL_BIN)
	python3 -m serial.tools.miniterm --eol LF --dtr 0 --rts 0 --filter direct $(K210-SERIALPORT) 115200

path:
	@echo $(BIN)
	@echo $(INCLUDE)
	@echo $(LIB)
	@echo $(GDB)
	@echo $(OPENOCD)

	@echo
	@$(BIN)/riscv64-unknown-elf-gcc --version 
	@echo
	@$(GDB) --version 
	@echo
	@$(OPENOCD) -f ./debug/k210.cfg

debug-s:
	@echo
	@$(OPENOCD) -f ./debug/k210.cfg

debug-c:
	@echo
	@$(GDB) $(KERNEL_ELF)

connect-serial:
	screen ${K210-SERIALPORT} 115200

.PHONY: build env kernel clean run-inner path doc debug-s debug-c connect-serial

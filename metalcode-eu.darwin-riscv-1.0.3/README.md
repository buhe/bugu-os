# GNU RISC-V toolchain for macOS

<div>
<img src="https://raw.githubusercontent.com/metalcode-eu/darwin-riscv/master/images/RISCV.png" alt="RISC-V" width="20%">
<img src="https://raw.githubusercontent.com/metalcode-eu/darwin-riscv/master/images/GNU.png" alt="GNU" width="20%">
<img src="https://raw.githubusercontent.com/metalcode-eu/darwin-riscv/master/images/macOS.png" alt="macOS" width="20%">
</div>

This repository is the original macOS version of the GNU Compiler for RISCV 
packaged for Visual Studio Code: 
[RISC-V Toolchain ](https://github.com/riscv/riscv-gnu-toolchain)

# Kendryte OpenOCD
The Open On-Chip Debugger version for the Kendryte K210 is included.

## Prerequisites
The RISC-V compiler depends on the following libraries. 

| library       | description                                                  |
|---------------|--------------------------------------------------------------|
| isl           | Integer Set Library for the polyhedral model                 |
| libmpc        | library for the arithmetic of high precision complex numbers |
| libusb        | library for USB access                                       |
| libusb-compat | library for USB-JTAG access                                  |

<img src="https://raw.githubusercontent.com/metalcode-eu/darwin-riscv/master/images/Homebrew.png" alt="Homebrew" width="10%" style="float: right;">

The Open On-Chip Debugger uses a USB debug probe. You need the USB-library 
installed on your Mac to use the debug probe. Use the Homebrew package manager 
to install the 'libusb' library.

See [Homebrew](https://brew.sh) for more information. 

> /usr/bin/ruby -e "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install)"  
> brew install libusb libusb-compat isl libmpc

# Install
In Visual Studio Code goto extensions (shift+cmd+x), search for '*metalcode-eu*'
and install the extension that is suited for your operating system. 

The extension adds a path to the Kendryte K210 openocd version. You can use in 
tasks.json.

- riscv.bin
- riscv.include
- riscv.lib
- kendryte.openocd

## Example tasks.json
```json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "build path",
      "type": "shell",
      "command": "make path",
      "options": {
        "env": {
          "BIN": "${config:riscv.bin}",
          "INCLUDE": "${config:riscv.include}",
          "LIB": "${config:riscv.lib}",
          "GDB": "${config:riscv.gdb}",
          "OPENOCD": "${config:kendryte.openocd}",
        }
      },
      "group": {
        "kind": "build",
        "isDefault": true,
      },
      "problemMatcher": "$gcc"
    }
  ]
}
```
## Example makefile 
```make
.PHONY: path

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
	@$(OPENOCD) --version 
```

# Version 1.0.3
Compiled OpenOCD Kendryte version with support for SiPEED USB-JTAG/TTL probe. 
- Kendryte Open On-Chip Debugger For RISC-V v0.2.3 (2019-02-21)

<img src="https://raw.githubusercontent.com/metalcode-eu/darwin-riscv/master/images/SiPEED_USB_JTAG_TTL.jpg" alt="SiPEED USB-JTAG/TTL" width="45%" float="left">
<img src="https://raw.githubusercontent.com/metalcode-eu/darwin-riscv/master/images/Sipeed_MAix_BiT.png" alt="SiPEED MAix-Bit" width="45%" float="right">

The OpenOCD configuration file for the Kendryte K210 connected to the MAix-BiT development board. 
```openocd
# SiPEED USB-JTAG/TTL 
interface ftdi
ftdi_device_desc "Dual RS232"
ftdi_vid_pid 0x0403 0x6010
ftdi_channel 0
ftdi_layout_init 0x0508 0x0f1b
ftdi_layout_signal nTRST -data 0x0200 -noe 0x0100
ftdi_layout_signal nSRST -data 0x0800 -noe 0x0400

transport select jtag
adapter_khz 3000

# server port
gdb_port 3333
telnet_port 4444

# add cpu target
set _CHIPNAME riscv
jtag newtap $_CHIPNAME cpu -irlen 5 -expected-id 0x04e4796b

set _TARGETNAME $_CHIPNAME.cpu
target create $_TARGETNAME riscv -chain-position $_TARGETNAME

# command
init
if {[ info exists pulse_srst]} {
  ftdi_set_signal nSRST 0
  ftdi_set_signal nSRST z
}
halt
```

# Version 1.0.2
This repository uses the following software versions:
- riscv64-unknown-elf-gcc (GCC) 9.2.0
- GNU gdb (GDB) 8.3.0.20190516-git
- Kendryte Open On-Chip Debugger For RISC-V v0.2.2 (2019-01-17)

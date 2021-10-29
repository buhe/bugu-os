#!/bin/sh
# Create symlinks for RISC-V toolchain for macOS
VERSION=1.0.2
PACKAGE=~/.vscode/extensions/metalcode-eu.darwin-riscv-${VERSION}
TOOLCHAIN=${PACKAGE}/bin
CROSS=riscv64-unknown-elf-

echo "Create symlinks for GNU Arm embedded toolchain for macOS"
echo ${PACKAGE}

ln -hfs ${TOOLCHAIN}/${CROSS}as                 /usr/local/bin/${CROSS}as
ln -hfs ${TOOLCHAIN}/${CROSS}gcc                /usr/local/bin/${CROSS}gcc
ln -hfs ${TOOLCHAIN}/${CROSS}g++                /usr/local/bin/${CROSS}g++
ln -hfs ${TOOLCHAIN}/${CROSS}ld                 /usr/local/bin/${CROSS}ld
ln -hfs ${TOOLCHAIN}/${CROSS}objcopy            /usr/local/bin/${CROSS}objcopy
ln -hfs ${TOOLCHAIN}/${CROSS}objdump            /usr/local/bin/${CROSS}objdump
ln -hfs ${TOOLCHAIN}/${CROSS}nm                 /usr/local/bin/${CROSS}nm
ln -hfs ${TOOLCHAIN}/${CROSS}strip              /usr/local/bin/${CROSS}strip
ln -hfs ${TOOLCHAIN}/${CROSS}size               /usr/local/bin/${CROSS}size
ln -hfs ${TOOLCHAIN}/${CROSS}readelf            /usr/local/bin/${CROSS}readelf
ln -hfs ${TOOLCHAIN}/${CROSS}gdb                /usr/local/bin/${CROSS}gdb
ln -hfs ${PACKAGE}/kendryte-openocd/bin/openocd /usr/local/bin/kendryte-openocd
echo
${CROSS}gcc --version 
echo
make --version 
echo
kendryte-openocd --version 
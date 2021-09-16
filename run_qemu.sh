#!/bin/bash

LOADER=./target/x86_64-unknown-uefi/release/loader.efi

mkdir -p mnt/EFI/BOOT/

cp $LOADER mnt/EFI/BOOT/BOOTX64.EFI

qemu-system-x86_64 \
   -drive if=pflash,format=raw,readonly,file=./OVMF/OVMF_CODE.fd \
   -drive if=pflash,format=raw,file=./OVMF/OVMF_VARS.fd \
   -drive format=raw,file=fat:rw:./mnt \
   -chardev stdio,mux=on,id=com1 \
   -serial chardev:com1

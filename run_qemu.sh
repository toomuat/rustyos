#!/bin/bash

LOADER=./target/x86_64-unknown-uefi/release/loader.efi

mkdir -p mnt/EFI/BOOT/

cp $LOADER mnt/EFI/BOOT/BOOTX64.EFI

QEMU_OPT="
    -drive if=pflash,format=raw,readonly,file=./OVMF/OVMF_CODE.fd \
    -drive if=pflash,format=raw,file=./OVMF/OVMF_VARS.fd \
    -drive format=raw,file=fat:rw:./mnt"

for OPT in "$@"
do
case $OPT in
    "--serial")
        # echo serial
        QEMU_OPT="${QEMU_OPT} \
        -chardev stdio,mux=on,id=com1 \
        -serial chardev:com1"
        break
        ;;
    "--monitor")
        # echo monitor
        QEMU_OPT="${QEMU_OPT} \
        -monitor stdio"
        break
        ;;
    *)
        ;;
esac
shift
done

# echo $QEMU_OPT
qemu-system-x86_64 $QEMU_OPT

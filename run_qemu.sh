#!/bin/bash

LOADER=./target/x86_64-unknown-uefi/release/loader.efi
KERNEL=./target/x86_64-unknown-rustyos/release/kernel.elf

mkdir -p mnt/EFI/BOOT/

cp $LOADER mnt/EFI/BOOT/BOOTX64.EFI
cp $KERNEL mnt/kernel.elf

QEMU_OPT="
    -drive if=pflash,format=raw,readonly,file=./OVMF/OVMF_CODE.fd \
    -drive if=pflash,format=raw,file=./OVMF/OVMF_VARS.fd \
    -drive format=raw,file=fat:rw:./mnt"

for OPT in "$@"
do
case $OPT in
    "--serial")
        # Output to serial console
        QEMU_OPT="${QEMU_OPT} \
        -chardev stdio,mux=on,id=com1 \
        -serial chardev:com1"
        break
        ;;
    "--monitor")
        # QEMU monitor
        QEMU_OPT="${QEMU_OPT} \
        -monitor stdio"
        break
        ;;
    "--cui")
        # Disable GUI
        QEMU_OPT="${QEMU_OPT} \
        -chardev stdio,mux=on,id=com1 \
        -serial chardev:com1 \
        -display none"
        echo cui
        break
        ;;
    *)
        ;;
esac
shift
done

# echo $QEMU_OPT
qemu-system-x86_64 $QEMU_OPT

#!/bin/bash

set -eu

LOADER=./target/x86_64-unknown-uefi/release/loader.efi
KERNEL=./target/x86_64-unknown-rustyos/release/kernel.elf

# Check OVMF_CODE.fd and OVMF_VARS.fd exists
OVMF_DIR="OVMF"
if [ -d "$OVMF_DIR" ]
then
    if [ "$(ls -A $OVMF_DIR)" ]; then
        # echo "Directory $OVMF_DIR is not Empty"
        :
    else
        cp edk2/Build/OvmfX64/DEBUG_GCC5/FV/OVMF_CODE.fd OVMF/
        cp edk2/Build/OvmfX64/DEBUG_GCC5/FV/OVMF_VARS.fd OVMF/
    fi
else
    echo "Directory $OVMF_DIR not found."
    exit 0
fi

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

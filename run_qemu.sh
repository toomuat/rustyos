#!/bin/bash

set -eu

LOADER=./target/x86_64-unknown-uefi/release/loader.efi
KERNEL=./target/x86_64-unknown-rustyos/release/kernel.elf
MOUNT_DIR=./mnt
OVMF_DIR=./OVMF
QEMU_OPT=

for OPT in "$@"
do
case $OPT in
    "--serial")
        # Output to serial console
        QEMU_OPT="-chardev stdio,mux=on,id=com1 \
        -serial chardev:com1"
        ;;
    "--monitor")
        # QEMU monitor
        QEMU_OPT="-monitor stdio"
        echo monitor
        ;;
    "--cui")
        # Disable GUI
        QEMU_OPT="-chardev stdio,mux=on,id=com1 \
        -serial chardev:com1 \
        -display none"
        ;;
    *)
        # When this script executed from cargo
        if [[ ${OPT} == *.efi ]]
        then
            LOADER=${OPT}
            MOUNT_DIR=../mnt
            OVMF_DIR=../OVMF
            # echo ${LOADER}
        elif [[ ${OPT} == *.elf ]]
        then
            KERNEL=${OPT}
            MOUNT_DIR=../mnt
            OVMF_DIR=../OVMF
        fi
        ;;
esac
shift
done

QEMU_OPT="${QEMU_OPT} \
    -drive if=pflash,format=raw,readonly,file=${OVMF_DIR}/OVMF_CODE.fd \
    -drive if=pflash,format=raw,file=${OVMF_DIR}/OVMF_VARS.fd \
    -drive format=raw,file=fat:rw:${MOUNT_DIR}"

# Check OVMF_CODE.fd and OVMF_VARS.fd exists
if [ -d "${OVMF_DIR}" ]
then
    if [ "$(ls -A ${OVMF_DIR})" ]; then
        # echo "Directory $OVMF_DIR is not Empty"
        :
    else
        cp edk2/Build/OvmfX64/DEBUG_GCC5/FV/OVMF_CODE.fd ${OVMF_DIR}
        cp edk2/Build/OvmfX64/DEBUG_GCC5/FV/OVMF_VARS.fd ${OVMF_DIR}
    fi
else
    echo "Directory ${OVMF_DIR} not found."
    exit 0
fi

mkdir -p ${MOUNT_DIR}/EFI/BOOT/

cp ${LOADER} ${MOUNT_DIR}/EFI/BOOT/BOOTX64.EFI
cp ${KERNEL} ${MOUNT_DIR}/kernel.elf

# echo $QEMU_OPT
qemu-system-x86_64 ${QEMU_OPT}

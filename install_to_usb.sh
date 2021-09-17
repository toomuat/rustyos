#!/bin/bash

LOADER=./target/x86_64-unknown-uefi/release/loader.efi
KERNEL=./target/x86_64-unknown-rustyos/release/kernel.elf
LOADER_SIZE=$(stat --format="%s" ${LOADER})
KERNEL_SIZE=$(stat --format="%s" ${KERNEL})
DISK_SIZE=$((${LOADER_SIZE} + ${KERNEL_SIZE}))
echo "Disk size: ${DISK_SIZE} byte"
SECTOR_END=$((2048 + (${DISK_SIZE} + 512) / 512 + 2))
SECTOR_END=$((${SECTOR_END} * 2))
echo "Sector end: ${SECTOR_END}"

# Format disk partition to FAT32
DEV="/dev/sdc"
DEV_PART=${DEV}"1"
sudo dd if=/dev/zero of=$DEV bs=512 count=1
sudo parted ${DEV} mklabel msdos
sudo parted ${DEV} "mkpart primary fat32 2048s ${SECTOR_END}s"
# sudo parted ${DEV} "mkpart p fat32 2048s -0"
sudo mkfs.vfat -c ${DEV_PART}

# Copy loader and kernel to mount point
MOUNT_DIR="/tmp/mnt"
mkdir -p ${MOUNT_DIR}
sudo mount ${DEV_PART} ${MOUNT_DIR}
sudo mkdir -p ${MOUNT_DIR}/EFI/BOOT
sudo cp ${LOADER} ${MOUNT_DIR}/EFI/BOOT/BOOTX64.EFI
sudo cp ${KERNEL} ${MOUNT_DIR}/

sudo hdparm --fibmap ${MOUNT_DIR}/$(basename $KERNEL)

sudo umount ${MOUNT_DIR}
rmdir ${MOUNT_DIR}
sync


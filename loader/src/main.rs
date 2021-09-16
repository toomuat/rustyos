#![no_std]
#![no_main]
#![feature(abi_efiapi)]

use uefi::prelude::*;
use uefi_services;

#[entry]
fn efi_main(image: Handle, mut st: SystemTable<Boot>) -> Status {
    todo!();
}

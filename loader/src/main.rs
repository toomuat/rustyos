#![no_std]
#![no_main]
#![feature(asm)]
#![feature(abi_efiapi)]

use core::fmt::Write;
use log::info;
use uefi::prelude::*;
use uefi_services;

#[entry]
fn efi_main(_image: Handle, mut st: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut st).unwrap_success();
    st.stdout().reset(false).unwrap_success();
    writeln!(st.stdout(), "Hello, World!").unwrap();

    let rev = st.uefi_revision();
    let (major, minor) = (rev.major(), rev.minor());
    info!("UEFI {}.{}", major, minor); // UEFI 2.70

    loop{
        unsafe {
            asm!("hlt");
        }
    }

    Status::SUCCESS
}

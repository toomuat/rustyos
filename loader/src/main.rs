#![no_std]
#![no_main]
#![feature(asm)]
#![feature(abi_efiapi)]

use core::fmt::Write;
use log::info;
use uefi::prelude::*;
use uefi::proto::media::file::{File, FileAttribute, FileInfo, FileMode, FileType};
use uefi::table::boot::MemoryType;
use uefi_services;

#[entry]
fn efi_main(image: Handle, mut st: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut st).unwrap_success();
    st.stdout().reset(false).unwrap_success();
    writeln!(st.stdout(), "Hello, World!").unwrap();

    let rev = st.uefi_revision();
    let (major, minor) = (rev.major(), rev.minor());
    info!("UEFI {}.{}", major, minor); // UEFI 2.70

    let bt = st.boot_services();
    let kernel_file = "kernel.elf";

    let kernel_entry_addr = load_kernel(kernel_file, image, bt);

    loop {
        unsafe {
            asm!("hlt");
        }
    }

    Status::SUCCESS
}

fn load_kernel(file_name: &str, image: Handle, bt: &BootServices) -> usize {
    // Open root directory
    let mut root_dir = {
        let sfs = bt.get_image_file_system(image).unwrap_success();
        unsafe { &mut *sfs.get() }.open_volume().unwrap_success()
    };

    // Open kernel file
    let mut kernel_file = match root_dir
        .open(file_name, FileMode::Read, FileAttribute::empty())
        .unwrap_success()
        .into_type()
        .unwrap_success()
    {
        FileType::Regular(file) => file,
        FileType::Dir(_) => panic!(),
    };

    // Get kernel file size
    let kernel_file_size = kernel_file
        .get_boxed_info::<FileInfo>()
        .unwrap_success()
        .file_size() as usize;
    info!("Kernel size: {}", kernel_file_size);

    let p = bt
        .allocate_pool(MemoryType::LOADER_DATA, kernel_file_size)
        .unwrap_success();

    // Read kernel file into the memory
    let mut buf = unsafe { core::slice::from_raw_parts_mut(p, kernel_file_size as usize) };

    kernel_file.read(&mut buf).unwrap_success();
    kernel_file.close();

    0
}

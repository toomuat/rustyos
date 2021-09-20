#![no_std]
#![no_main]
#![feature(asm)]
#![feature(abi_efiapi)]

#[macro_use]
extern crate alloc;

use common::graphics::FrameBuffer;
use core::fmt::Write;
use core::slice;
use goblin::elf;
use log::info;
use uefi::prelude::*;
use uefi::proto::console::gop::GraphicsOutput;
use uefi::proto::media::file::{File, FileAttribute, FileInfo, FileMode, FileType};
use uefi::table::boot::{AllocateType, MemoryDescriptor, MemoryType};
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

    // Load kernel elf file
    let kernel_file = "kernel.elf";
    let kernel_entry_addr = load_kernel(kernel_file, image, bt);

    let entry_pointer = unsafe { kernel_entry_addr } as *const ();
    let kernel_entry = unsafe {
        core::mem::transmute::<
            *const (),
            extern "sysv64" fn(
                fb: *mut FrameBuffer,
                mi: *mut uefi::proto::console::gop::ModeInfo,
            ) -> (),
        >(entry_pointer)
    };

    // Get frame buffer
    let gop = bt.locate_protocol::<GraphicsOutput>().unwrap_success();
    let gop = unsafe { &mut *gop.get() };

    let mut mi = gop.current_mode_info();
    let mut fb = gop.frame_buffer();
    let fb_pt = fb.as_mut_ptr(); // FrameBuffer.base
    let fb_size = fb.size();
    info!("Frame buffer size: {}", fb_size);

    let mut fb = FrameBuffer {
        base: fb_pt,
        size: fb_size,
    };

    st.stdout().reset(false).unwrap_success();

    // Exit boot service
    let max_mmap_size =
        st.boot_services().memory_map_size() + 8 * core::mem::size_of::<MemoryDescriptor>();
    let mut mmap_storage = vec![0; max_mmap_size].into_boxed_slice();
    let (st, _iter) = st
        .exit_boot_services(image, &mut mmap_storage[..])
        .expect_success("Failed to exit boot services");

    kernel_entry(unsafe { &mut fb as *mut FrameBuffer }, unsafe {
        &mut mi as *mut uefi::proto::console::gop::ModeInfo
    });

    Status::SUCCESS
}

fn load_kernel(file_name: &str, image: Handle, bt: &BootServices) -> u64 {
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

    parse_elf(buf, bt)
}

fn parse_elf(buf: &[u8], bt: &BootServices) -> u64 {
    let elf = elf::Elf::parse(&buf).expect("Failed to parse ELF");

    let mut dest_start = usize::MAX;
    let mut dest_end = 0;
    for ph in elf.program_headers.iter() {
        if ph.p_type != elf::program_header::PT_LOAD {
            continue;
        }
        dest_start = dest_start.min(ph.p_vaddr as usize);
        dest_end = dest_end.max((ph.p_vaddr + ph.p_memsz) as usize);

        info!("dest_start: 0x{:x}", dest_start);
        info!("dest_end: 0x{:x}", dest_end);
    }

    const PAGE_SIZE: usize = 0x1000;
    info!(
        "Kernel page count: {}",
        (dest_end - dest_start + PAGE_SIZE - 1) / PAGE_SIZE
    );

    bt.allocate_pages(
        AllocateType::Address(dest_start),
        MemoryType::LOADER_DATA,
        (dest_end - dest_start + PAGE_SIZE - 1) / PAGE_SIZE,
    )
    .expect_success("Failed to allocate pages for kernel");

    for ph in elf.program_headers.iter() {
        if ph.p_type != elf::program_header::PT_LOAD {
            continue;
        }
        let ofs = ph.p_offset as usize;
        let fsize = ph.p_filesz as usize;
        let msize = ph.p_memsz as usize;
        let dest = unsafe { slice::from_raw_parts_mut(ph.p_vaddr as *mut u8, msize) };
        dest[..fsize].copy_from_slice(&buf[ofs..ofs + fsize]);
        dest[fsize..].fill(0);
    }

    info!("ELF entry: 0x{:x}", elf.entry);

    elf.entry
}

#![no_std]
#![no_main]
#![feature(abi_efiapi)]
#![feature(vec_into_raw_parts)]

#[macro_use]
extern crate alloc;

mod graphics;
mod memory;

use alloc::vec::Vec;
use core::fmt::Write;
use core::slice;
use goblin::elf;
use graphics::FrameBuffer;
use log::info;
use uefi::prelude::*;
use uefi::proto::console::gop::GraphicsOutput;
use uefi::proto::media::file::{File, FileAttribute, FileInfo, FileMode, FileType};
use uefi::table::boot::{AllocateType, MemoryType};
use uefi::table::cfg::ACPI_GUID;
use uefi::CStr16;

#[entry]
fn efi_main(image: Handle, mut st: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut st).unwrap();
    st.stdout().reset(false).unwrap();
    writeln!(st.stdout(), "Hello, World! in efi_main").unwrap();

    let rev = st.uefi_revision();
    let (major, minor) = (rev.major(), rev.minor());
    info!("UEFI {}.{}", major, minor); // UEFI 2.70

    let bt = st.boot_services();

    // Get memory map
    dump_memory_map(image, bt);

    let rsdp = st
        .config_table()
        .iter()
        .find(|config| config.guid == ACPI_GUID)
        .map(|config| config.address as u64)
        .expect("Could not find RSDP");
    info!("RSDP: 0x{:x}", rsdp);

    // Load kernel elf file
    let kernel_file = cstr16!("kernel.elf");
    let kernel_entry_addr = load_kernel(kernel_file, image, bt);

    let entry_pointer = kernel_entry_addr as *const ();
    let kernel_entry = unsafe {
        core::mem::transmute::<
            *const (),
            extern "sysv64" fn(
                fb: *mut FrameBuffer,
                mi: *mut uefi::proto::console::gop::ModeInfo,
                mm: &memory::MemoryMap,
                rsdp: u64,
            ) -> (),
        >(entry_pointer)
    };

    // Get frame buffer
    let gop = bt.locate_protocol::<GraphicsOutput>().unwrap();
    let gop = unsafe { &mut *gop.get() };

    let mut mi = gop.current_mode_info();
    let mut fb = gop.frame_buffer();
    let fb_pt = fb.as_mut_ptr(); // FrameBuffer.base
    let fb_size = fb.size();
    info!("Frame buffer size: {}", fb_size);
    info!("Mode info: {:?}", mi);

    let mut fb = FrameBuffer {
        base: fb_pt,
        size: fb_size,
    };

    // Exit boot service
    let sizes = st.boot_services().memory_map_size();
    let max_mmap_size = sizes.map_size + 8 * sizes.entry_size;
    let mut mmap_buf = vec![0; max_mmap_size].into_boxed_slice();
    let mut descriptors = Vec::with_capacity(max_mmap_size);
    info!("st.exit_boot_services");
    let (_st, memory_descriptor) = st
        .exit_boot_services(image, &mut mmap_buf[..])
        .expect("Failed to exit boot services");

    for d in memory_descriptor {
        // Unified Extensible Firmware Interface (UEFI) Specification, version 2.8
        // 7.2 Memory Allocation Services
        // Check available memory after calling exit boot services
        if matches!(
            d.ty,
            MemoryType::CONVENTIONAL
                | MemoryType::BOOT_SERVICES_CODE
                | MemoryType::BOOT_SERVICES_DATA
        ) {
            descriptors.push(memory::MemoryDescriptor {
                phys_start: d.phys_start,
                page_count: d.page_count,
            });
        }
    }

    let (ptr, len, _) = descriptors.into_raw_parts();
    // descriptors_len is used to cast descriptors pointer to slice
    let memory_map = memory::MemoryMap {
        descriptors: ptr as *const memory::MemoryDescriptor,
        descriptors_len: len as u64,
    };

    kernel_entry(
        &mut fb as *mut FrameBuffer,
        &mut mi as *mut uefi::proto::console::gop::ModeInfo,
        &memory_map,
        rsdp,
    );

    Status::SUCCESS
}

fn dump_memory_map(image: Handle, bt: &BootServices) {
    let sizes = bt.memory_map_size();
    let enough_mmap_size = sizes.map_size + 8 * sizes.entry_size;
    let mut mmap_buf = vec![0; enough_mmap_size];
    let (_, descriptors) = bt
        .memory_map(&mut mmap_buf)
        .expect("Failed to retrieve UEFI memory map");

    // Open root directory
    let mut root_dir = {
        let sfs = bt.get_image_file_system(image).unwrap();
        unsafe { &mut *sfs.interface.get() }.open_volume().unwrap()
    };

    let file_name = cstr16!("mem_map");
    let attr = FileAttribute::empty();
    let status = root_dir
        .open(file_name, FileMode::CreateReadWrite, attr)
        .expect("Failed to create file")
        .into_type()
        .unwrap();
    let mut file = match status {
        FileType::Regular(file) => file,
        FileType::Dir(_) => panic!("Not a regular file: {}", file_name),
    };

    file.write("Index, Type, Type(name), PhysicalStart, NumberOfPages, Attribute\n".as_bytes())
        .unwrap();

    for (i, d) in descriptors.enumerate() {
        file.write(
            format!(
                "{}, {:x}, {:?}, {:08x}, {:x}, {:x}\n",
                i,
                d.ty.0,
                d.ty,
                d.phys_start,
                d.page_count,
                d.att.bits() & 0xfffff
            )
            .as_bytes(),
        )
        .unwrap();
    }

    file.close();
}

fn load_kernel(file_name: &CStr16, image: Handle, bt: &BootServices) -> u64 {
    // Open root directory
    let mut root_dir = {
        let sfs = bt.get_image_file_system(image).unwrap();
        unsafe { &mut *sfs.interface.get() }.open_volume().unwrap()
    };

    // Open kernel file
    let mut kernel_file = match root_dir
        .open(file_name, FileMode::Read, FileAttribute::empty())
        .expect("Failed to open kernel")
        .into_type()
        .unwrap()
    {
        FileType::Regular(file) => file,
        FileType::Dir(_) => panic!(),
    };

    // Get kernel file size
    let kernel_file_size = kernel_file
        .get_boxed_info::<FileInfo>()
        .unwrap()
        .file_size() as usize;
    // info!("Kernel size: {}", kernel_file_size);

    let p = bt
        .allocate_pool(MemoryType::LOADER_DATA, kernel_file_size)
        .unwrap();

    // Read kernel file into the memory
    let mut buf = unsafe { core::slice::from_raw_parts_mut(p, kernel_file_size as usize) };

    kernel_file.read(&mut buf).unwrap();
    kernel_file.close();

    parse_elf(buf, bt)
}

fn parse_elf(buf: &[u8], bt: &BootServices) -> u64 {
    let elf = elf::Elf::parse(buf).expect("Failed to parse ELF");

    let mut dest_start = usize::MAX;
    let mut dest_end = 0;
    for ph in elf.program_headers.iter() {
        if ph.p_type != elf::program_header::PT_LOAD {
            continue;
        }
        dest_start = dest_start.min(ph.p_vaddr as usize);
        dest_end = dest_end.max((ph.p_vaddr + ph.p_memsz) as usize);

        // info!("dest_start: 0x{:x}", dest_start);
        // info!("dest_end: 0x{:x}", dest_end);
    }

    const PAGE_SIZE: usize = 0x1000;
    // info!(
    //     "Kernel page count: {}",
    //     (dest_end - dest_start + PAGE_SIZE - 1) / PAGE_SIZE
    // );

    bt.allocate_pages(
        AllocateType::Address(dest_start),
        MemoryType::LOADER_DATA,
        (dest_end - dest_start + PAGE_SIZE - 1) / PAGE_SIZE,
    )
    .expect("Failed to allocate pages for kernel");

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

    // info!("ELF entry: 0x{:x}", elf.entry);

    elf.entry
}

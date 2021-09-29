use core::slice;

use crate::println;

const PAGE_SIZE: usize = 0x1000;

pub struct MemoryDescriptor {
    pub phys_start: u64,
    pub page_count: u64,
}

#[derive(Debug)]
pub struct MemoryMap {
    pub descriptors: *const MemoryDescriptor,
    pub descriptors_len: u64,
}

pub fn initialize(mm: &MemoryMap) {
    let (phys_start, phys_end) = max_available_memory_area(mm);
    println!("0x{:x}, 0x{:x}", phys_start, phys_end);
}

// Return maximum contiguous available memory area
// Memory map to calculate this area is passed from UEFI bootloader
fn max_available_memory_area(mm: &MemoryMap) -> (u64, u64) {
    let descriptors = unsafe { slice::from_raw_parts(mm.descriptors, mm.descriptors_len as usize) };

    let mut phys_start: u64 = descriptors[0].phys_start;
    let mut phys_end: u64 =
        descriptors[0].phys_start + descriptors[0].page_count * PAGE_SIZE as u64;
    let mut mem_range = phys_end - phys_start;

    let mut phys_start_tmp = phys_start;
    let mut phys_end_tmp = phys_end;
    let mut mem_range_tmp;

    for d in descriptors {
        if phys_end_tmp == d.phys_start {
            // Available memory area is contiguous
            phys_end_tmp = d.phys_start + d.page_count * PAGE_SIZE as u64;
            mem_range_tmp = phys_end_tmp - phys_start_tmp;

            // Curent contiguous available memory area is larger than previous one
            if mem_range_tmp > mem_range {
                mem_range = mem_range_tmp;
                phys_start = phys_start_tmp;
                phys_end = phys_end_tmp;

                // println!("0x{:x}, 0x{:x}", phys_start, phys_end);
            }
        } else {
            phys_start_tmp = d.phys_start;
            phys_end_tmp = d.phys_start + d.page_count * PAGE_SIZE as u64;
        }

        // println!("0x{:x}, 0x{:x}", d.phys_start, phys_end_tmp);
    }

    (phys_start, phys_end)
}

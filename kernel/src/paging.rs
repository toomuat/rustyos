pub struct MemoryDescriptor {
    pub phys_start: u64,
    pub page_count: u64,
}

#[derive(Debug)]
pub struct MemoryMap {
    pub descriptors: *const MemoryDescriptor,
    pub descriptors_len: u64,
}

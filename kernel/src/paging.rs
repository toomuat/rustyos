

#[repr(align(4096))]
pub struct PageTable {
    entries: [u64; 512],
    // entries: [PageTableEntry; 512],
}


pub fn test() {
    use x86_64::registers::control::Cr3;
    use byteorder::LittleEndian;
    use crate::serial;

    let (level_4_page_table, _) = Cr3::read();
    // println!("Level 4 page table at: {:?}", level_4_page_table.start_address());

    // let i: i64 = 12345;
    // let mut bs = [0u8; core::mem::size_of::<i64>()];
    // bs.as_mut()
    //     .write_i64::<LittleEndian>(i)
    //     .expect("Unable to write");

    // for i in bs {
    //     // println!("{:X}", i);
    //     serial::write_byte(i);
    // }

    // let var1 = 12345678_u64;
    // let raw_bytes: [u8; 8] = unsafe { core::mem::transmute(var1) };
    // for byte in &raw_bytes {
    //     // println!("{}", byte);
    //     serial::write_byte(*byte);
    // }

    // let i: u64 = 66;
    // let ip: *const u64 = &i;
    // let bp: *const u8 = ip as *const _;
    // let bs: &[u8] = unsafe { core::slice::from_raw_parts(bp, core::mem::size_of::<u64>()) };

    // for i in bs {
    //     // println!("{:X}", i);
    //     serial::write_byte(*i);
    // }

    let ptr = 0xdeadbeaf as *mut u32;
    unsafe { *ptr = 42; }
}

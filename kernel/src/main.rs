#![no_std]
#![no_main]
#![feature(asm)]
#![feature(lang_items)]

mod graphics;
mod interrupt;
mod serial;

use core::panic::PanicInfo;
use graphics::{FrameBuffer, ModeInfo};

#[no_mangle]
extern "C" fn kernel_main(fb: *mut FrameBuffer, mi: *mut ModeInfo, rsdp: u64, proc_number: u32) {
    if proc_number != 0 {
        // AP

        serial::write_byte('C' as u8);
        // serial::write_byte(proc_number as u8);
        serial::write_byte('\n' as u8);

        loop {
            unsafe {
                asm!("hlt");
            }
        }
    }

    serial::initialize();
    serial::write_byte('A' as u8);
    serial::write_str("Hello serial\n");

    interrupt::disable();
    interrupt::enable();

    let hor_res = unsafe { (*mi).hor_res } as usize;

    for i in 0..(hor_res / 2) {
        for j in 0..(hor_res / 2) {
            unsafe {
                (*fb).base.add((i + hor_res * j) * 4).write_volatile(255);
                (*fb).base.add((i + hor_res * j) * 4 + 1).write_volatile(0);
                (*fb).base.add((i + hor_res * j) * 4 + 2).write_volatile(0);
            }
        }
    }

    loop {
        unsafe {
            asm!("hlt");
        }
    }
}

#[lang = "eh_personality"]
fn eh_personality() {}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}

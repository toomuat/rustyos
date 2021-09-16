#![no_std]
#![no_main]
#![feature(asm)]
#![feature(lang_items)]

mod graphics;

use core::panic::PanicInfo;
use graphics::{FrameBuffer, ModeInfo};

#[no_mangle]
extern "C" fn kernel_main(fb: *mut FrameBuffer, mi: *mut ModeInfo) {
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

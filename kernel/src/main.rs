#![no_std]
#![no_main]
#![feature(asm)]
#![feature(lang_items)]

pub mod serial;

use core::panic::PanicInfo;
use serial::{init_serial, write_serial, write_str_serial};

#[no_mangle]
extern "C" fn kernel_main() {
    init_serial();
    write_serial('A' as u8);
    write_str_serial("Hello serial\n");

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

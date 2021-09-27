#![no_std]
#![no_main]
#![feature(asm)]
#![feature(lang_items)]
#![feature(abi_x86_interrupt)]
#![feature(in_band_lifetimes)]

mod gdt;
mod graphics;
mod interrupt;
mod serial;

use core::panic::PanicInfo;
use graphics::{FrameBuffer, ModeInfo};

#[no_mangle]
extern "C" fn kernel_main(fb: *mut FrameBuffer, mi: *mut ModeInfo, _rsdp: u64) {
    interrupt::disable();

    serial::initialize();
    serial::write_byte('A' as u8);
    serial::write_str("Hello serial\n");

    graphics::initialize(fb, mi);

    gdt::initialize();
    interrupt::init();

    if interrupt::check_apic() {
        serial::write_str("CPU supports APIC\n");
    }

    interrupt::enable();

    loop {
        unsafe {
            asm!("hlt");
        }
    }
}

#[lang = "eh_personality"]
fn eh_personality() {}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

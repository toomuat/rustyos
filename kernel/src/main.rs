#![no_std]
#![no_main]
#![feature(asm)]
#![feature(lang_items)]
#![feature(abi_x86_interrupt)]
#![feature(in_band_lifetimes)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

mod gdt;
mod graphics;
mod interrupt;
mod paging;
mod serial;

use core::panic::PanicInfo;
use graphics::{FrameBuffer, ModeInfo};

#[no_mangle]
extern "C" fn kernel_main(
    fb: *mut FrameBuffer,
    mi: *mut ModeInfo,
    mm: &paging::MemoryMap,
    _rsdp: u64,
) {
    interrupt::disable();

    serial::initialize();
    serial::write_byte(b'A');
    serial::write_str("Hello serial\n");

    graphics::initialize(fb, mi);

    paging::initialize(mm);

    gdt::initialize();
    interrupt::init();

    if interrupt::check_apic() {
        serial::write_str("CPU supports APIC\n");
    }

    interrupt::enable();

    #[cfg(test)]
    test_main();

    // panic!("testpanic");

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
    println!("{:?}", info);
    loop {}
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
}

#[test_case]
fn trivial_assertion() {
    print!("trivial assertion... ");
    assert_eq!(1, 1);
    println!("[ok]");
}

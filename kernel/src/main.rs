#![no_std]
#![no_main]
#![feature(asm)]
#![feature(lang_items)]

pub mod ascii_font;
pub mod console;
pub mod graphics;
pub mod log;
pub mod serial;

use crate::log::*;
// extern crate alloc;

// use alloc::format;
use console::Console;
use core::panic::PanicInfo;
use graphics::{FrameBuffer, Graphics, ModeInfo, PixelColor};
use serial::{init_serial, write_serial, write_str_serial};

const BG_COLOR: PixelColor = PixelColor(0, 80, 80);
const FG_COLOR: PixelColor = PixelColor(255, 128, 0);

fn initialize(fb: *mut FrameBuffer, mi: *mut ModeInfo) {
    unsafe { Graphics::initialize_instance(fb, mi) }
    Console::initialize(&FG_COLOR, &BG_COLOR);
    Graphics::instance().clear(&BG_COLOR);
}

#[no_mangle]
extern "C" fn kernel_main(fb: *mut FrameBuffer, mi: *mut ModeInfo) {
    // initialize(fb, mi);

    // for i in 0..25 {
    //     for j in 0..25 {
    //         unsafe {
    //             // (*fb).write_value(i + j * (*mi).stride as usize, [0, 80, 80]);

    //             (*fb).write_byte(j * 3 + i * 3 * (*mi).stride as usize, 255);
    //             (*fb).write_byte(j * 3 + 1 + i * 3 * (*mi).stride as usize, 255);
    //             (*fb).write_byte(j * 3 + 2 + i * 3 * (*mi).stride as usize, 255);
    //         }
    //     }
    // }

    let hor_res = unsafe { (*mi).hor_res };
    for i in 0..80 as usize {
        unsafe {
            (*fb).write_byte(
                i * (*mi).stride as usize + 80 * (*mi).stride as usize * 2,
                255,
            );
            (*fb).write_byte(
                i * (*mi).stride as usize + 1 + 80 * (*mi).stride as usize * 2,
                255,
            );
            (*fb).write_byte(
                i * (*mi).stride as usize + 2 + 80 * (*mi).stride as usize * 2,
                255,
            );

            // (*fb).write_byte(i + 2 * (*mi).hor_res as usize * 3, 255);
            // (*fb).write_byte(i + 2 * (*mi).stride as usize * (*mi).hor_res as usize, 255);
        }
    }

    init_serial();
    write_serial('A' as u8);
    write_str_serial("Hello serial\n");

    let res = unsafe { (*mi).resolution() };
    // let s = format!("Frame resolution: ({}, {})\n", res.0, res.1);
    // write_str_serial(&s);

    info!("Hello kernel main");

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

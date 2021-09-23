use x86_64::instructions::port::*;

const PORT: u16 = 0x3f8;

pub fn initialize() {
    unsafe {
        u8::write_to_port(PORT + 1, 0x00);
        u8::write_to_port(PORT + 3, 0x80);
        u8::write_to_port(PORT + 0, 0x03);
        u8::write_to_port(PORT + 1, 0x00);
        u8::write_to_port(PORT + 3, 0x03);
        u8::write_to_port(PORT + 2, 0xc7);
        u8::write_to_port(PORT + 4, 0x0b);
        u8::write_to_port(PORT + 4, 0x1e);
        u8::write_to_port(PORT + 0, 0xae);

        // Check if serial is faulty (i.e: not same byte as sent)
        if u8::read_from_port(PORT + 0) != 0xae {
            return;
        } else {
            // ok
        }

        // If serial is not faulty set it in normal operation mode
        // (not-loopback with IRQs enabled and OUT#1 and OUT#2 bits enabled)
        u8::write_to_port(PORT + 4, 0xf);
    }
}

fn is_transmit_empty() -> u8 {
    unsafe { u8::read_from_port(PORT + 5) & 0x20 }
}

pub fn write_byte(c: u8) {
    while is_transmit_empty() == 0 {}
    unsafe {
        u8::write_to_port(PORT, c);
    }
}

pub fn write_str(s: &str) {
    let ss = s.as_bytes();
    for i in 0..s.len() {
        write_byte(ss[i]);
    }
}

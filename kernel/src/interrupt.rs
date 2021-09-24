use lazy_static::lazy_static;
use x86_64::instructions::interrupts;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

const T_IRQ0: u8 = 0x20;

const IRQ_TIMER: u8 = 0;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.double_fault.set_handler_fn(double_fault_handler);

        idt[(T_IRQ0 + IRQ_TIMER) as usize].set_handler_fn(timer_handler);
        idt
    };
}

pub fn init() {
    IDT.load();
    unsafe {
        disable_pic_8259();
    }
}

unsafe fn disable_pic_8259() {
    use x86_64::instructions::port::*;

    u8::write_to_port(0xa1, 0xff);
    u8::write_to_port(0x21, 0xff);
}

pub fn enable() {
    interrupts::enable();
}

pub fn disable() {
    interrupts::disable();
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    // panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
    loop {}
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    // panic!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
    loop {}
}

pub extern "x86-interrupt" fn timer_handler(_stack_frame: InterruptStackFrame) {
    disable();

    use crate::serial;
    serial::write_byte('*' as u8);

    enable();
}

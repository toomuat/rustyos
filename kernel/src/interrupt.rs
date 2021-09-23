use x86_64::instructions::interrupts;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

pub fn enable() {
    interrupts::enable();
}

pub fn disable() {
    interrupts::disable();
}

unsafe fn disable_pic_8259() {
    use x86_64::instructions::port::*;

    u8::write_to_port(0xa1, 0xff);
    u8::write_to_port(0x21, 0xff);
}

static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

pub unsafe fn initialize() {
    initialize_idt();
    disable_pic_8259();
}

const EXTERNAL_IRQ_OFFSET: u32 = 32; // first 32 entries are reserved by CPU
const IRQ_TIMER: u32 = 0;

unsafe fn initialize_idt() {
    // IDT.breakpoint.set_handler_fn(breakpoint_handler);
    // IDT.page_fault.set_handler_fn(page_fault_handler);
    // IDT.double_fault
    //     .set_handler_fn(double_fault_handler)
    //     .set_stack_index(DOUBLE_FAULT_IST_INDEX);
    // IDT[(EXTERNAL_IRQ_OFFSET + IRQ_TIMER) as usize].set_handler_fn(timer_handler);
    // IDT[(EXTERNAL_IRQ_OFFSET + IRQ_KBD) as usize].set_handler_fn(kbd_handler);
    // IDT[(EXTERNAL_IRQ_OFFSET + IRQ_COM1) as usize].set_handler_fn(com1_handler);
    IDT.load();
}

// extern "x86-interrupt" fn timer_handler(_stack_frame: InterruptStackFrame) {
//     let msg = Message::Timer;
//     let _ = message_queue().enqueue(msg);
//     unsafe { LAPIC.wait().set_eoi(0) };
// }

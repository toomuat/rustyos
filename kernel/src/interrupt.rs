use crate::{println, serial};
use lazy_static::lazy_static;
use raw_cpuid::CpuId;
use x86_64::instructions::interrupts;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

const T_IRQ0: u8 = 0x20;

const IRQ_TIMER: u8 = 0;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.double_fault.set_handler_fn(double_fault_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);

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

    const PIC1: u16 = 0x20; // IO base address for master PIC
    const PIC2: u16 = 0xA0; // IO base address for slave PIC
    const PIC1_DATA: u16 = PIC1 + 1;
    const PIC2_DATA: u16 = PIC2 + 1;

    u8::write_to_port(PIC1_DATA, 0xFF);
    u8::write_to_port(PIC2_DATA, 0xFF);
}

pub fn check_apic() -> bool {
    let mut apic_supported = false;
    let cpuid = CpuId::new();

    if let Some(vf) = cpuid.get_vendor_info() {
        assert!(vf.as_str() == "GenuineIntel" || vf.as_str() == "AuthenticAMD");
    }

    cpuid.get_feature_info().map(|finfo| {
        if finfo.has_apic() {
            apic_supported = true;
        }
    });

    apic_supported
}

pub fn enable() {
    interrupts::enable();
}

pub fn disable() {
    interrupts::disable();
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT\n{:#?}", stack_frame);
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    loop {}
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn timer_handler(_stack_frame: InterruptStackFrame) {
    disable();

    serial::write_byte('*' as u8);

    enable();
}

#[allow(dead_code)]
pub fn check_double_fault() {
    unsafe {
        *(0xdeadbeefff as *mut u64) = 42;
    };
}

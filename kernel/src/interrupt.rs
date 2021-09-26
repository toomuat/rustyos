use crate::serial;
use apic;
use lazy_static::lazy_static;
use raw_cpuid::CpuId;
use volatile::{ReadWrite, WriteOnly};
// use volatile::access::{ReadWrite, WriteOnly};
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

// Switching from the legacy 8259 PIC to the modern APIC
// https://georgeclaghorn.com/2020/08/8259-pic-to-apic/

#[repr(C)]
pub struct APIC {
    _1: [u32; 44],
    end_of_interrupt_register: WriteOnly<u32>,
    _2: [u32; 155],
    timer_vector_register: ReadWrite<u32>,
    _3: [u32; 23],
    timer_initial_count_register: ReadWrite<u32>,
    _4: [u32; 23],
    timer_divide_configuration_register: ReadWrite<u32>,
}

impl APIC {
    pub unsafe fn get() -> &'static mut APIC {
        &mut *(0xFEE00000 as *mut APIC)
    }

    pub fn initialize(&mut self) {
        self.timer_vector_register
            .write(0x20000 | (T_IRQ0 + IRQ_TIMER) as u32);

        self.timer_divide_configuration_register.write(0b1011);
        self.timer_initial_count_register.write(200000000);
        // self.timer_initial_count_register.write(10000000);

        self.end_of_interrupt_register.write(0);
    }

    pub fn complete(&mut self) {
        self.end_of_interrupt_register.write(0);
    }
}

use spin::Mutex;

lazy_static! {
    static ref LAPIC: Mutex<&'static mut APIC> = Mutex::new(unsafe { APIC::get() });
}

pub fn init() {
    IDT.load();
    unsafe {
        disable_pic_8259();
    }
    // let apic = unsafe { apic::ApicBase::new(0xFEE00000 as *mut ()) };
    // apic.timer_local_vector_table_entry()
    //     .read()
    //     .set_timer_value(0x00020000 | (T_IRQ0 + IRQ_TIMER) as u32);

    LAPIC.lock().initialize();
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

    serial::write_byte('*' as u8);

    // EOI
    LAPIC.lock().complete();

    enable();
}

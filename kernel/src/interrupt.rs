use crate::serial;
// use apic;
use lazy_static::lazy_static;
use raw_cpuid::CpuId;
use spin::Mutex;
use x86_64::instructions::interrupts;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

const T_IRQ0: u8 = 0x20;
const IRQ_TIMER: u8 = 0;
const APIC_BASE: u32 = 0xFEE00000;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.double_fault.set_handler_fn(double_fault_handler);

        idt[(T_IRQ0 + IRQ_TIMER) as usize].set_handler_fn(timer_handler);
        idt
    };
}

#[repr(C)]
pub struct APIC {
    _researved1: [u32; 2],
    id: u32,
    version: u32,
    _researved2: [u32; 4],
    task_priority: u32,
    arbitation_priority: u32,
    processor_priority: u32,
    end_of_interrupt: u32,
    remote_read: u32,
    logical_destination: u32,
    destination_format: u32,
    spurious_interrupt_vector: u32,
    in_service: [u32; 8],
    trigger_mode: [u32; 8],
    interrupt_request: [u32; 8],
    error_status: u32,
    _researved3: [u32; 5],
    lvt_corrected_machine_check_interrupt: u32,
    interrupt_command: u32,
    lvt_timer: u32,
    lvt_thermal_sensor: u32,
    lvt_performance_monitoring_counters: u32,
    lvt_lint0: u32,
    lvt_lint1: u32,
    lvt_error: u32,
    initial_counter: u32,
    current_count: u32,
    _researved4: [u32; 4],
    divide_configuration: u32,
    _researved5: u32,
}

impl APIC {
    pub unsafe fn get() -> &'static mut APIC {
        &mut *(APIC_BASE as *mut APIC)
    }

    pub fn initialize(&mut self) {
        self.lapic_write(
            Offset::TimerLocalVectorTableEntry,
            0x20000 | (T_IRQ0 + IRQ_TIMER) as u32,
        );
        self.lapic_write(Offset::TimerDivideConfiguration, 0b1011);
        self.lapic_write(Offset::TimerInitialCount, 200000000);
        self.eoi();
    }

    pub fn eoi(&mut self) {
        self.lapic_write(Offset::EndOfInterrupt, 0);
    }

    pub fn lapic_write(&mut self, index: Offset, value: u32) {
        unsafe {
            core::ptr::write_volatile((APIC_BASE + index as u32) as *mut u32, value);
        }
    }
}

#[repr(usize)]
pub enum Offset {
    _Id = 0x20,
    _Version = 0x30,
    _TaskPriority = 0x80,
    _ArbitrationPriority = 0x90,
    _ProcessorPriority = 0xa0,
    EndOfInterrupt = 0xb0,
    _RemoteRead = 0xc0,
    _LocalDestination = 0xd0,
    _DestinationFormat = 0xe0,
    _SpuriousInterruptVector = 0xf0,
    _InService = 0x100,
    _TriggerMode = 0x180,
    _InterruptRequest = 0x200,
    _ErrorStatus = 0x280,
    _InterruptCommand = 0x300,
    TimerLocalVectorTableEntry = 0x320,
    _ThermalLocalVectorTableEntry = 0x330,
    _PerformanceCounterLocalVectorTableEntry = 0x340,
    _LocalInterrupt0VectorTableEntry = 0x350,
    _LocalInterrupt1VectorTableEntry = 0x360,
    _ErrorVectorTableEntry = 0x370,
    TimerInitialCount = 0x380,
    _TimerCurrentCount = 0x390,
    TimerDivideConfiguration = 0x3e0,
    _ExtendedApicFeature = 0x400,
    _ExtendedApicControl = 0x410,
    _SpecificEndOfInterrupt = 0x420,
    _InterruptEnable = 0x480,
    _ExtendedInterruptLocalVectorTable = 0x500,
}

lazy_static! {
    static ref LAPIC: Mutex<&'static mut APIC> = Mutex::new(unsafe { APIC::get() });
}

pub fn init() {
    IDT.load();
    unsafe {
        disable_pic_8259();
    }
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
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    // panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
    loop {}
}

extern "x86-interrupt" fn breakpoint_handler(_stack_frame: InterruptStackFrame) {
    // panic!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
    loop {}
}

pub extern "x86-interrupt" fn timer_handler(_stack_frame: InterruptStackFrame) {
    disable();

    serial::write_byte('*' as u8);

    LAPIC.lock().eoi();

    enable();
}

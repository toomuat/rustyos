use x86_64::instructions::segmentation::{Segment, CS, SS};
use x86_64::instructions::tables::load_tss;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

static mut TSS: TaskStateSegment = TaskStateSegment::new();
static mut GDT: GlobalDescriptorTable = GlobalDescriptorTable::new();

pub fn initialize() {
    unsafe {
        TSS.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };

        let code_selector = GDT.add_entry(Descriptor::kernel_code_segment());
        let data_selector = GDT.add_entry(Descriptor::kernel_data_segment());
        let tss_selector = GDT.add_entry(Descriptor::tss_segment(&TSS));
        GDT.load();

        CS::set_reg(code_selector);
        SS::set_reg(data_selector);
        load_tss(tss_selector);
    }
}

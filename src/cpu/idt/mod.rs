//! Defines the Interrupt Descriptor Table that the kernel will use.

mod exceptions;
mod pic;
mod syscall;

use crate::utility::instr::{lidt, DescriptorTablePointer};

use super::gdt::KERNEL_CODE_SEGMENT;

/// The global IDT that the kernel will use.
///
/// This array must be properly initialized before it can be used.
static mut IDT: [u64; 256] = [0; 256];

/// The pointer to the IDT that will be loaded with `lidt`.
static IDTP: DescriptorTablePointer = DescriptorTablePointer {
    limit: 256 * 8 - 1,
    base: unsafe { core::ptr::addr_of!(IDT) as *mut () },
};

/// The offset of the PIC.
///
/// The following 32 interrupts (32 to 63) are reserved for the PIC.
pub const PIC_OFFSET: u8 = 32;

/// The stack frame that is pushed onto the stack when an interrupt is triggered.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct InterruptStackFrame {
    /// The value of the instruction pointer at the time of the interrupt.
    pub ip: u32,
    /// The value of the code segment register at the time of the interrupt.
    pub cs: u32,
    /// The flags at the time of the interrupt.
    pub flags: u32,
    /// The value of the stack pointer at the time of the interrupt.
    pub sp: u32,
    /// The value of the stack segment register at the time of the interrupt.
    pub ss: u32,
}

/// Initializes the IDT.
///
/// # Safety
///
/// The IDT must not be currently in use.
pub fn init() {
    unsafe {
        IDT[0] = create_gate_descriptor(false, exceptions::division_error as usize);
        IDT[1] = create_gate_descriptor(false, exceptions::debug as usize);
        IDT[2] = create_gate_descriptor(false, exceptions::non_maskable_interrupt as usize);
        IDT[3] = create_gate_descriptor(true, exceptions::breakpoint as usize);
        IDT[4] = create_gate_descriptor(false, exceptions::overflow as usize);
        IDT[5] = create_gate_descriptor(false, exceptions::bound_range_exceeded as usize);
        IDT[6] = create_gate_descriptor(false, exceptions::invalid_opcode as usize);
        IDT[7] = create_gate_descriptor(false, exceptions::device_not_available as usize);
        IDT[8] = create_gate_descriptor(false, exceptions::double_fault as usize);
        IDT[10] = create_gate_descriptor(false, exceptions::invalid_tss as usize);
        IDT[11] = create_gate_descriptor(false, exceptions::segment_not_present as usize);
        IDT[12] = create_gate_descriptor(false, exceptions::stack_segment_fault as usize);
        IDT[13] = create_gate_descriptor(false, exceptions::general_protection_fault as usize);
        IDT[14] = create_gate_descriptor(false, exceptions::page_fault as usize);
        IDT[16] = create_gate_descriptor(false, exceptions::x87_floating_point as usize);
        IDT[17] = create_gate_descriptor(false, exceptions::alignment_check as usize);
        IDT[18] = create_gate_descriptor(false, exceptions::machine_check as usize);
        IDT[19] = create_gate_descriptor(false, exceptions::simd_floating_point as usize);
        IDT[20] = create_gate_descriptor(false, exceptions::virtualization as usize);
        IDT[21] = create_gate_descriptor(false, exceptions::control_protection as usize);
        IDT[28] = create_gate_descriptor(false, exceptions::hypervisor_injection as usize);
        IDT[29] = create_gate_descriptor(false, exceptions::vmm_communication as usize);
        IDT[30] = create_gate_descriptor(false, exceptions::security_exception as usize);

        IDT[32] = create_gate_descriptor(true, pic::timer as usize);
        IDT[33] = create_gate_descriptor(true, pic::keyboard as usize);
        IDT[34] = create_gate_descriptor(true, pic::cascade as usize);
        IDT[35] = create_gate_descriptor(true, pic::com2 as usize);
        IDT[36] = create_gate_descriptor(true, pic::com1 as usize);
        IDT[37] = create_gate_descriptor(true, pic::lpt2 as usize);
        IDT[38] = create_gate_descriptor(true, pic::floppy as usize);
        IDT[39] = create_gate_descriptor(true, pic::lpt1 as usize);
        IDT[40] = create_gate_descriptor(true, pic::rtc as usize);
        IDT[41] = create_gate_descriptor(true, pic::periph1 as usize);
        IDT[42] = create_gate_descriptor(true, pic::periph2 as usize);
        IDT[43] = create_gate_descriptor(true, pic::periph3 as usize);
        IDT[44] = create_gate_descriptor(true, pic::mouse as usize);
        IDT[45] = create_gate_descriptor(true, pic::fpu as usize);
        IDT[46] = create_gate_descriptor(true, pic::ata1 as usize);
        IDT[47] = create_gate_descriptor(true, pic::ata2 as usize);

        IDT[0x80] = create_gate_descriptor(false, syscall::system_call as usize);

        lidt(&IDTP);
    }
}

/// Creates a gate descriptor suitable for the IDT.
fn create_gate_descriptor(is_interrupt: bool, handler: usize) -> u64 {
    let mut val = 0;

    // offset_1
    val |= handler as u64 & 0xFFFF;
    // segment_selector
    val |= (KERNEL_CODE_SEGMENT as u64) << 16;
    // reserved
    val |= 0 << 32;
    // gateType
    val |= if is_interrupt { 0xE } else { 0xF } << 40;
    // dpl
    val |= 0 << 45;
    // present
    val |= 1 << 47;
    // offset_2
    val |= ((handler as u64 >> 16) & 0xFFFF) << 48;

    val
}

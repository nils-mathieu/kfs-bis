//! Defines the interrupt service routines for CPU exceptions.

use core::arch::asm;

use bitflags::bitflags;

use super::InterruptStackFrame;

pub extern "x86-interrupt" fn division_error(_stack_frame: InterruptStackFrame) {
    panic!("Received a DIVISION_ERROR fault.");
}

pub extern "x86-interrupt" fn debug(_stack_frame: InterruptStackFrame) {
    panic!("Received a DEBUG fault/trap.");
}

pub extern "x86-interrupt" fn non_maskable_interrupt(_stack_frame: InterruptStackFrame) {
    panic!("Received a NON_MASKABLE_INTERRUPT interrupt.");
}

pub extern "x86-interrupt" fn breakpoint(_stack_frame: InterruptStackFrame) {
    panic!("Received a BREAKPOINT trap.");
}

pub extern "x86-interrupt" fn overflow(_stack_frame: InterruptStackFrame) {
    panic!("Received an OVERFLOW trap.");
}

pub extern "x86-interrupt" fn bound_range_exceeded(_stack_frame: InterruptStackFrame) {
    panic!("Received a BOUND_RANGE_EXCEEDED fault.");
}

pub extern "x86-interrupt" fn invalid_opcode(_stack_frame: InterruptStackFrame) {
    panic!("Received an INVALID_OPCODE fault.");
}

pub extern "x86-interrupt" fn device_not_available(_stack_frame: InterruptStackFrame) {
    panic!("Received a DEVICE_NOT_AVAILABLE fault.");
}

pub extern "x86-interrupt" fn double_fault(
    _stack_frame: InterruptStackFrame,
    _error_code: u32,
) -> ! {
    panic!("Received a DOUBLE_FAULT fault.");
}

pub extern "x86-interrupt" fn invalid_tss(_stack_frame: InterruptStackFrame, error_code: u32) {
    panic!(
        "Received an INVALID_TSS fault with error code {:#x}.",
        error_code
    );
}

pub extern "x86-interrupt" fn segment_not_present(
    _stack_frame: InterruptStackFrame,
    error_code: u32,
) {
    panic!(
        "Received a SEGMENT_NOT_PRESENT fault with error code {:#x}.",
        error_code
    );
}

pub extern "x86-interrupt" fn stack_segment_fault(
    _stack_frame: InterruptStackFrame,
    error_code: u32,
) {
    panic!(
        "Received a STACK_SEGMENT_FAULT fault with error code {:#x}.",
        error_code
    );
}

pub extern "x86-interrupt" fn general_protection_fault(
    frame: InterruptStackFrame,
    error_code: u32,
) {
    panic!(
        "\
        Received a GENERAL_PROTECTION_FAULT fault with error code {:#x}.\n\
        > EIP = {:#x}\n\
        > ESP = {:#x}\
        ",
        error_code, frame.ip, frame.sp,
    );
}

bitflags! {
    /// The error code received with a page fault.
    #[derive(Clone, Copy, Debug)]
    #[repr(transparent)]
    pub struct PageFaultError: u32 {
        /// Whether the page was present.
        const PRESENT = 1 << 0;
        /// Whether the page was written to. Otherwise, it was read from.
        const WRITE = 1 << 1;
        /// Whether the fault occured while the CPU was in ring 3.
        const USER = 1 << 2;
        /// One of the page directory or page table entries was malformed and included
        /// some reserved bits.
        const RESERVED_WRITE = 1 << 3;
        /// Whether the fault was caused by an instruction fetch.
        const INSTRUCTION_FETCH = 1 << 4;
    }
}

pub extern "x86-interrupt" fn page_fault(frame: InterruptStackFrame, error_code: PageFaultError) {
    let mut cr2: usize;
    unsafe {
        asm!("mov {}, cr2", out(reg) cr2, options(nostack, nomem, preserves_flags));
    }

    panic!(
        "\
        Received a PAGE_FAULT fault.\n\
        > ERROR   = {:?}\n\
        > EIP     = {:#x}\n\
        > ESP     = {:#x}\n\
        > ADDRESS = {:#x}\
        ",
        error_code, frame.ip, frame.sp, cr2,
    );
}

pub extern "x86-interrupt" fn x87_floating_point(_stack_frame: InterruptStackFrame) {
    panic!("Received an X87_FLOATING_POINT fault.");
}

pub extern "x86-interrupt" fn alignment_check(_stack_frame: InterruptStackFrame, error_code: u32) {
    panic!(
        "Received an ALIGNMENT_CHECK fault with error code {:#x}.",
        error_code
    );
}

pub extern "x86-interrupt" fn machine_check(_stack_frame: InterruptStackFrame) -> ! {
    panic!("Received a MACHINE_CHECK fault.");
}

pub extern "x86-interrupt" fn simd_floating_point(_stack_frame: InterruptStackFrame) {
    panic!("Received an SIMD_FLOATING_POINT fault.");
}

pub extern "x86-interrupt" fn virtualization(_stack_frame: InterruptStackFrame) {
    panic!("Received a VIRTUALIZATION fault.");
}

pub extern "x86-interrupt" fn control_protection(
    _stack_frame: InterruptStackFrame,
    error_code: u32,
) {
    panic!(
        "Received a CONTROL_PROTECTION_EXCEPTION fault with error code {:#x}.",
        error_code
    );
}

pub extern "x86-interrupt" fn hypervisor_injection(_stack_frame: InterruptStackFrame) {
    panic!("Received a HYPERVISOR_INJECTION_EXCEPTION fault.");
}

pub extern "x86-interrupt" fn vmm_communication(
    _stack_frame: InterruptStackFrame,
    error_code: u32,
) {
    panic!(
        "Received a VMM_COMMUNICATION_EXCEPTION fault with erro code {:#x}.",
        error_code
    );
}

pub extern "x86-interrupt" fn security_exception(
    _stack_frame: InterruptStackFrame,
    error_code: u32,
) {
    panic!(
        "Received a SECURITY_EXCEPTION fault with error code {:#x}.",
        error_code
    );
}

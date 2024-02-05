//! Defines the Global Descriptor Table that the kernel will use.

use core::arch::asm;

use crate::log;
use crate::utility::instr::{lgdt, DescriptorTablePointer};

/// The address at which the GDT must be loaded.
const ADDRESS: *mut u64 = 0x800 as *mut u64;

/// The offset of the kernel data segment within the kernel's GDT.
pub const KERNEL_DATA_SEGMENT: u16 = 0x10;
/// The offset of the kernel code segment within the kernel's GDT.
pub const KERNEL_CODE_SEGMENT: u16 = 0x08;

/// The GDT that will be copied and loaded.
const GDT: [u64; 5] = [
    // Null Segment
    0u64,
    // Kernel Mode Code Segment
    0x00cf9a000000ffff,
    // Kernel Mode Data Segment
    0x00cf92000000ffff,
    // User Mode Code Segment
    0x00cffa000000ffff,
    // User Mode Data Segment
    0x00cff2000000ffff,
];

/// The GDTP that will be loaded with `lgdt`.
const GDTP: DescriptorTablePointer = DescriptorTablePointer {
    limit: 5 * 8 - 1,
    base: ADDRESS as *mut (),
};

/// Installs the kernel's GDT.
///
/// # Safety
///
/// The memory address where the GDT is installed must not currently be in use.
pub unsafe fn init() {
    core::ptr::copy_nonoverlapping(GDT.as_ptr(), ADDRESS, GDT.len());

    lgdt(&GDTP);

    // Reload the data segment registers.
    asm!(
        "
        mov {tmp:x}, {data_segment_offset}
        mov ds, {tmp:x}
        mov es, {tmp:x}
        mov fs, {tmp:x}
        mov gs, {tmp:x}
        mov ss, {tmp:x}
        ",
        tmp = lateout(reg) _,
        data_segment_offset = const KERNEL_DATA_SEGMENT,
        options(preserves_flags, nostack, nomem)
    );

    // Reload the code segment register.
    // Note: because it's not possible to directly modify the CS register (like above),
    // we have to use a far jump.
    asm!(
        "jmp ${code_segment_offset}, $2f; 2:",
        code_segment_offset = const KERNEL_CODE_SEGMENT,
        options(att_syntax)
    );
}

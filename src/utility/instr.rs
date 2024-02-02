//! Common CPU instructions.

use core::arch::asm;

/// Writes a value to the specified I/O port.
///
/// # Safety
///
/// Writing to arbitrary I/O ports can compromise memory safety.
#[inline(always)]
pub unsafe fn outb(port: u16, value: u8) {
    asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
}

/// Reads a value from the specified I/O port.
///
/// # Safety
///
/// Reading from arbitrary I/O ports can compromise memory safety.
#[inline(always)]
pub unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    asm!("in al, dx", in("dx") port, out("al") value, options(nomem, nostack, preserves_flags));
    value
}

/// Clears the interrupt-enable flag.
#[inline(always)]
pub fn cli() {
    unsafe {
        asm!("cli", options(nomem, nostack, preserves_flags));
    }
}

/// Halts the CPU until the next interrupt arrives.
#[inline(always)]
pub fn hlt() {
    unsafe {
        asm!("hlt", options(nomem, nostack, preserves_flags));
    }
}

/// A pointer to a descriptor table.
#[derive(Debug, Clone, Copy)]
#[repr(packed, C)]
pub struct DescriptorTablePointer {
    pub limit: u16,
    pub base: u32,
}

/// Loads a new global descriptor table.
#[inline(always)]
pub unsafe fn lgdt(gdt: &DescriptorTablePointer) {
    unsafe {
        asm!("lgdt [{}]", in(reg) gdt, options(nomem, nostack, preserves_flags));
    }
}

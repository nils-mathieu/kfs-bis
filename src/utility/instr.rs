//! Common CPU instructions.

use core::arch::asm;

use bitflags::bitflags;

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

/// Sets the interrupt-enable flag.
#[inline(always)]
pub fn sti() {
    unsafe {
        asm!("sti", options(nomem, nostack, preserves_flags));
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

/// Loads a new interrupt descriptor table.
#[inline(always)]
pub unsafe fn lidt(idt: &DescriptorTablePointer) {
    unsafe {
        asm!("lidt [{}]", in(reg) idt, options(nomem, nostack, preserves_flags));
    }
}

/// Reads the current value of the RFLAGS register.
pub fn read_rflags() -> RFlags {
    let mut value: u32;
    unsafe {
        asm!("pushf; pop {}", out(reg) value, options(nomem, preserves_flags));
    }
    RFlags::from_bits_retain(value)
}

bitflags! {
    /// The flags in the RFLAGS register.
    #[derive(Default, Clone, Copy)]
    pub struct RFlags: u32 {
        const CARRY = 1 << 0;
        const PARITY = 1 << 2;
        const ADJUST = 1 << 4;
        const ZERO = 1 << 6;
        const SIGN = 1 << 7;
        const TRAP = 1 << 8;
        const INTERRUPT = 1 << 9;
        const DIRECTION = 1 << 10;
        const OVERFLOW = 1 << 11;
        const IOPL0 = 0b00 << 12;
        const IOPL1 = 0b01 << 12;
        const IOPL2 = 0b10 << 12;
        const IOPL3 = 0b11 << 12;
        const NESTED_TASK = 1 << 14;
        const RESUME = 1 << 16;
        const VIRTUAL8086 = 1 << 17;
        const ALIGNMENT_CHECK = 1 << 18;
        const VIRTUAL_INTERRUPT = 1 << 19;
        const VIRTUAL_INTERRUPT_PENDING = 1 << 20;
        const ID = 1 << 21;
    }
}

//! This module provides definitions of the types defined in the multiboot protocol specification.

use bitflags::bitflags;

/// The magic number that the bootloader uses to determine whether the kernel is
/// multiboot-compliant.
pub const HEADER_MAGIC: u32 = 0x1BADB002;

/// The magic number that the bootloader will load into the EAX register upon entry to the kernel.
pub const EAX_MAGIC: u32 = 0x2BADB002;

/// The multiboot header that the bootloader will read in the kernel's binary file.
#[repr(C)]
#[derive(Debug)]
pub struct Header {
    /// The value '0x1BADB002'.
    pub magic: u32,
    /// A combination of flags that the bootloader will use to determine the features that the
    /// kernel wants.
    pub flags: HeaderFlags,
    /// A checksum. When added to `magic` and `flags`, the result must be a 32-bit value 0.
    pub checksum: u32,
}

impl Header {
    /// Creates a new multiboot header with the given flags.
    pub const fn new(flags: HeaderFlags) -> Self {
        Self {
            magic: HEADER_MAGIC,
            flags,
            checksum: HEADER_MAGIC.wrapping_add(flags.bits()).wrapping_neg(),
        }
    }
}

bitflags! {
    /// A bunch of flags representating the features that the kernel requests from the bootloader.
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy)]
    pub struct HeaderFlags: u32 {
        /// Requests the bootloader to align all loaded modules on a page (4KiB) boundary.
        const ALIGN_MODULES = 1 << 0;
        /// Requests the bootloader to provide information about the memory map.
        const MEMORY_MAP = 1 << 1;
    }
}

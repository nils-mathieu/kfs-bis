//! This module provides definitions of the types defined in the multiboot protocol specification.

use core::ffi::c_char;
use core::fmt::Debug;

use bitflags::bitflags;

use crate::die;

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

/// Information that the bootloader will provide to the kernel.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct MultibootInfo {
    /// A bunch of flags.
    pub flags: InfoFlags,
    /// The amount of lower memory available, in kilobytes.
    ///
    /// Lower memory starts at address 0 and ends at address 1 MiB. The maximum value for this
    /// field is 640 KiB.
    ///
    /// This is only available when bit 0 of `flags` is set.
    pub mem_lower: u32,
    /// The amount of upper memory available, in kilobytes.
    ///
    /// Upper memory starts at address 1 MiB.
    ///
    /// This is only available when bit 0 of `flags` is set.
    pub mem_upper: u32,
    /// The boot device that the bootloader loaded the kernel from.
    ///
    /// If the bootloader did not load the Kernel from a BIOS disk, this field is not available.
    ///
    /// The boot device is layed out as follows:
    ///
    /// +--------------+--------------+--------------+--------------+
    /// | 31 - 24      | 23 - 16      | 15 - 8       | 7 - 0        |
    /// +--------------+--------------+--------------+--------------+
    /// | part3        | part2        | part1        | drive number |
    /// +--------------+--------------+--------------+--------------+
    ///
    /// This field is only available when bit 1 of `flags` is set.
    pub boot_device: u32,
    /// The command line that the bootloader passed to the kernel.
    ///
    /// This is the physical address of a null-terminated string.
    ///
    /// This field is only available when bit 2 of `flags` is set.
    pub cmdline: *const c_char,
    /// The number of boot modules loaded by the bootloader.
    ///
    /// This is only available when bit 3 of `flags` is set, but note that this field might still
    /// be 0 even if bit 3 is set.
    pub mods_count: u32,
    /// The physical address of the first module structure. Subsequent module structures are
    /// located at increasing addresses.
    ///
    /// This is only available when bit 3 of `flags` is set.
    pub mods_addr: *mut Module,
    pub _syms: [u32; 4],
    /// The number of bytes in the memory map provided by the bootloader.
    ///
    /// This is only set when bit 6 of `flags` is set.
    pub mmap_length: u32,
    /// The address of the first entry in the memory map provided by the bootloader. Subsequent
    /// entries are located at increasing addresses.
    ///
    /// This is only set when bit 6 of `flags` is set.
    ///
    /// # Iteration
    ///
    /// This pointer point to the first entry in the list, but in order to get from one entry to
    /// the next, the size of the entry must be added to the pointer.
    pub mmap_addr: *mut MemMapEntry,
    pub _drives_length: u32,
    pub _drives_addr: u32,
    pub _config_table: u32,
    /// The name of the bootloader that loaded the kernel.
    ///
    /// This is a null-terminated C-like string.
    ///
    /// This is only present if the `flags` field has bit 9 set.
    pub bootloader_name: *const c_char,
}

bitflags! {
    /// A bunch of flags that indicate which fields of [`Info`] have been filled by the
    /// bootloader.
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy)]
    pub struct InfoFlags: u32 {
        /// Whether the `mem_lower` and `mem_upper` fields are set.
        const MEMORY = 1 << 0;
        /// Whether the `boot_device` field is set.
        const BOOT_DEVICE = 1 << 1;
        /// Whether the `cmdline` field is set.
        const CMDLINE = 1 << 2;
        /// Whether the `mods_count` and `mods_addr` fields are set.
        const MODULES = 1 << 3;
        /// Whether the `mmap_length` and `mmap_addr` fields are set.
        const MEMORY_MAP = 1 << 6;
        /// Whether the `bootloader_name` field is set.
        const BOOTLOADER_NAME = 1 << 9;
    }
}

/// Information about a loaded boot module.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct Module {
    /// The base physical address of the module.
    pub mod_start: u32,
    /// The end physical address of the module.
    pub mod_end: u32,
    /// A pointer to a string that represents the command line that the bootloader passed to the
    /// module.
    pub string: *const c_char,
    /// A reserved field.
    pub _reserved: u32,
}

/// An entry in the memory map.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MemMapEntry {
    /// The size of the structure, not including this field.
    pub size: u32,
    /// The lower 32 bits of the starting address of the memory region.
    pub addr_low: u32,
    /// The higher 32 bits of the starting address of the memory region.
    pub addr_high: u32,
    /// The lower 32 bits of the length of the memory region.
    pub len_low: u32,
    /// The higher 32 bits of the length of the memory region.
    pub len_high: u32,
    /// The type of the memory region.
    pub ty: MemMapType,
}

/// The type of the memory map entry.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct MemMapType(pub u32);

impl MemMapType {
    /// The memory region is available for general purpose use.
    pub const AVAILABLE: MemMapType = MemMapType(1);
    /// The memory region is useable but holds ACPI information.
    pub const ACPI_RECLAIMABLE: MemMapType = MemMapType(3);
    /// Memory that must be preserved when the system is hibernated or suspended.
    pub const PRESERVED: MemMapType = MemMapType(4);
    /// The memory region is defective and should not be used.
    pub const DEFECTIVE: MemMapType = MemMapType(5);
}

/// Returns an iterator over the memory map entries.
///
/// # Arguments
///
/// - `addr`: The value of the `mmap_addr` field in the multiboot info structure.
///
/// - `length`: The value of the `mmap_length` field of the multiboot info structure.
///
/// # Safety
///
/// The provided arguments must be valid as specified in the multiboot protocol. The memory
/// they reference must remain valid and borrowed for the lifetime `'a`.
pub unsafe fn iter_memory_map<'a>(
    addr: *const MemMapEntry,
    length: u32,
) -> impl Clone + Iterator<Item = &'a MemMapEntry> {
    let mut cur = addr;
    let mut total_offset = 0usize;

    core::iter::from_fn(move || {
        if total_offset >= length as usize {
            return None;
        }

        // Make sure that the cursor is properly
        // aligned.
        if !cur.is_aligned() {
            die("found a mis-aligned memory map entry");
        }

        let ret = &*cur;

        let skip_size = ret.size as usize + 4;
        total_offset += skip_size;
        cur = cur.byte_add(skip_size);

        Some(ret)
    })
}

use crate::multiboot;
use crate::utility::ArrayVec;

/// Stores information about the system.
pub struct SystemInfo {
    /// The memory map of the system.
    pub memory_map: ArrayVec<MemoryMapEntry, 16>,
}

/// An entry in the memory map.
#[derive(Debug, Clone, Copy)]
pub struct MemoryMapEntry {
    /// The base address of the memory region.
    pub base: u64,
    /// The length of the memory region.
    pub length: u64,
    /// The type of the memory region.
    pub ty: MemoryMapType,
}

impl MemoryMapEntry {
    /// Converts the provided [`multiboot::MemMapEntry`] into an kernel-specific [`MemoryMapEntry`].
    pub fn from_multiboot(e: &multiboot::MemMapEntry) -> Self {
        let base = e.addr_low as u64 | (e.addr_high as u64) << 32;
        let length = e.len_low as u64 | (e.len_high as u64) << 32;
        let ty = MemoryMapType::from_multiboot(e.ty);
        Self { base, length, ty }
    }
}

/// The type of a [`MemoryMapEntry`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryMapType {
    /// The memory is available for general purpose use.
    Available,
    /// The memory region is available but holds information about the ACPI tables.
    AcpiReclaimable,
    /// The memory is reserved for some purpose and cannot be used.
    Reserved,
    /// The memory is defective and should not be used.
    Defective,
    /// The memory must be preserved when the system is hibernated or suspended.
    Preserved,
}

impl MemoryMapType {
    /// Converts the provided [`multiboot::MemMapType`] into a [`MemoryMapType`].
    pub fn from_multiboot(t: multiboot::MemMapType) -> Self {
        match t {
            multiboot::MemMapType::AVAILABLE => Self::Available,
            multiboot::MemMapType::ACPI_RECLAIMABLE => Self::Available,
            multiboot::MemMapType::DEFECTIVE => Self::Defective,
            multiboot::MemMapType::PRESERVED => Self::Preserved,

            // The multiboot protocol indicates that unknown memory map types
            // should be treated as 'reserved' regions.
            _ => Self::Reserved,
        }
    }
}

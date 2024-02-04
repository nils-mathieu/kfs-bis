use crate::utility::ArrayVec;

/// Stores information about the system.
pub struct SystemInfo {
    /// The total amount of memory available, in bytes.
    pub total_memory: u32,
    /// The name of the bootloader.
    pub bootloader_name: Option<ArrayVec<u8, 62>>,
}

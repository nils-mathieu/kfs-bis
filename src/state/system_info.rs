use core::sync::atomic::AtomicU32;

use crate::utility::ArrayVec;

/// Stores information about the system.
pub struct SystemInfo {
    /// The total amount of memory available, in bytes.
    pub total_memory: u32,
    /// The name of the bootloader.
    pub bootloader_name: Option<ArrayVec<u8, 62>>,
    /// The total number of ticks since the system was started.
    ///
    /// If a tick is a millisecond, this value will overflow after 49.7 days.
    pub tick_count: AtomicU32,
}

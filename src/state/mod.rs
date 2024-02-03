//! Defines the structures used in the kernel's global state.

mod system_info;

use crate::utility::OnceCell;

pub use self::system_info::*;

/// The global state of the kernel.
///
/// This structure is meant to be shared across the entire kernel.
pub struct Global {
    /// Information about the system.
    pub system_info: SystemInfo,
}

/// The global state of the kernel.
pub static GLOBAL: OnceCell<Global> = OnceCell::new();
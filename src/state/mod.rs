//! Defines the structures used in the kernel's global state.

mod allocator;
mod process;
mod system_info;
mod user;

use crate::utility::Mutex;
use crate::utility::OnceCell;

pub use self::allocator::*;
pub use self::process::*;
pub use self::system_info::*;
pub use self::user::*;

/// The global state of the kernel.
///
/// This structure is meant to be shared across the entire kernel.
pub struct Global {
    /// Information about the system.
    pub system_info: SystemInfo,
    /// The physical memory allocator.
    pub allocator: Mutex<Allocator>,
}

/// The global state of the kernel.
pub static GLOBAL: OnceCell<Global> = OnceCell::new();

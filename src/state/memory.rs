use core::fmt::Display;

/// An error that occurs when memory cannot be allocated.
#[derive(Debug)]
pub struct OutOfMemory;

impl Display for OutOfMemory {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "out of memory")
    }
}

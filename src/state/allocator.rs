use core::fmt::Display;
use core::mem::MaybeUninit;

/// A physical page allocator.
///
/// This allocator operates on a page granularity.
pub struct Allocator {
    /// The list of pages that are available for allocation.
    pages: &'static mut [MaybeUninit<u32>],
    /// The number of pages that are available.
    len: usize,
}

impl Allocator {
    /// Creates a new [`Allocator`] with the provided backing storage.
    pub fn new(storage: &'static mut [MaybeUninit<u32>]) -> Self {
        Self {
            pages: storage,
            len: 0,
        }
    }

    /// Deallocates the provided page.
    ///
    /// # Validity
    ///
    /// It is not directly unsafe to deallocate a page that was never allocated
    /// in the first place, but it is probably a logic error.
    ///
    /// This might cause invalid behavior if the page is not actually available
    /// as it might be allocated later on by another part of the kernel.
    ///
    /// # Panics
    ///
    /// This function panics if the allocator has not enough memory to store the
    /// page.
    pub fn deallocate(&mut self, page: u32) {
        assert!(
            self.len < self.pages.len(),
            "out of memory for the allocator"
        );

        unsafe {
            self.pages.get_unchecked_mut(self.len).write(page);
        }

        self.len += 1;
    }

    /// Allocates a page and returns its physical address.
    #[inline]
    pub fn allocate(&mut self) -> Result<u32, OutOfMemory> {
        if self.len == 0 {
            return Err(OutOfMemory);
        }

        self.len -= 1;

        Ok(unsafe { self.pages.get_unchecked(self.len).assume_init() })
    }

    /// Returns the total amount of tracked memory, in bytes.
    #[inline]
    pub fn remaining_memory(&self) -> usize {
        self.len * 0x1000
    }
}

/// An error that occurs when memory cannot be allocated.
#[derive(Debug)]
pub struct OutOfMemory;

impl Display for OutOfMemory {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "out of memory")
    }
}

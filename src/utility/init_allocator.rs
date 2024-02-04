use core::alloc::Layout;
use core::mem::MaybeUninit;

use crate::oom;
use crate::state::OutOfMemory;

/// The init allocator is responsible for allocating initial memory for the kernel (before paging is
/// even enabled).
///
/// It cannot deallocate anything, meaning that any memory it gives out is forever lost and
/// cannot be reclaimed.
pub struct InitAllocator {
    /// The top of the stack that's used to allocate memory.
    ///
    /// This is the pointer that moves.
    top: usize,
    /// The base pointer of the memory stack.
    ///
    /// When `top` reaches this value, no more memory is available.
    base: usize,
}

impl InitAllocator {
    /// Creates a new [`InitAllocator`] instance.
    ///
    /// # Safety
    ///
    /// The provided `base` and `top` pointers must represent a block of memory
    /// that is now *owned* by the created [`InitAllocator`]. Until that instance
    /// is destroyed, the memory referenced cannot be used from outside of its
    /// allocation.
    ///
    /// It is possible to reclaim the remaining memory once the allocator is no
    /// longer in use.
    ///
    /// The memory that *is* allocated, however, must remain mapped to the same
    /// location for the entire duration of the kernel's lifetime.
    #[inline]
    pub unsafe fn new(base: usize, top: usize) -> Self {
        Self { base, top }
    }

    /// Attempts to allocate some memory.
    ///
    /// # Errors
    ///
    /// This function returns [`OutOfMemory`] if the allocator is out of memory.
    ///
    /// # Returns
    ///
    /// A raw pointer to the allocated address.
    pub fn try_allocate_raw(&mut self, layout: Layout) -> Result<*mut u8, OutOfMemory> {
        let mut addr = self.top;
        addr = addr.checked_sub(layout.size()).ok_or(OutOfMemory)?;
        addr &= !(layout.align() - 1);

        if addr < self.base {
            return Err(OutOfMemory);
        }

        self.top = addr;

        Ok(addr as *mut u8)
    }

    /// Allocates some memory and returns the allocated address.
    ///
    /// # Dies
    ///
    /// This function makes the kernel die if not enough memory is available.
    pub fn allocate_raw(&mut self, layout: Layout) -> *mut u8 {
        self.try_allocate_raw(layout).unwrap_or_else(|_| oom())
    }

    /// Allocates an instance of `T`.
    #[inline]
    pub fn allocate<T>(&mut self) -> &'static mut MaybeUninit<T> {
        unsafe { &mut *self.allocate_raw(Layout::new::<T>()).cast() }
    }

    /// Allocates a slice of `T`s.
    pub fn allocate_slice<T>(&mut self, count: usize) -> &'static mut [MaybeUninit<T>] {
        let layout = Layout::array::<T>(count).unwrap_or_else(|_| oom());
        let addr = self.allocate_raw(layout) as *mut MaybeUninit<T>;
        unsafe { core::slice::from_raw_parts_mut(addr, count) }
    }

    /// Returns the top address of the stack.
    #[inline(always)]
    pub fn top(&self) -> usize {
        self.top
    }

    /// Returns the base address of the stack.
    #[inline(always)]
    pub fn base(&self) -> usize {
        self.base
    }
}

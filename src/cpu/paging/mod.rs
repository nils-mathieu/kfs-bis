//! This module provides some ways to manipulate a page table and an address space.

mod address_space;
mod model;

use core::alloc::Layout;
use core::arch::asm;

use crate::die::oom;
use crate::state::OutOfMemory;
use crate::utility::InitAllocator;

pub use self::address_space::*;
pub use self::model::*;

/// Initiates paging and memory protection for the kernel.
pub unsafe fn init(allocator: &mut InitAllocator, upper_bound: u32) {
    struct InitContext<'a> {
        allocator: &'a mut InitAllocator,
    }

    unsafe impl<'a> Context for InitContext<'a> {
        #[inline]
        fn allocate(&mut self) -> Result<u32, OutOfMemory> {
            let layout = unsafe { Layout::from_size_align_unchecked(4096, 4096) };
            self.allocator
                .try_allocate_raw(layout)
                .map(|addr| addr as u32)
        }

        unsafe fn deallocate(&mut self, _: u32) {
            unreachable!("this Context implementation should never be used to deallocate pages");
        }

        #[inline]
        unsafe fn map(&self, physical: u32) -> *mut u8 {
            // At this point in the execution, we are setting up the kernel's address space, meaning
            // that paging is not yet initiating. Every "virtual" address is equal to its
            // physical address.
            physical as *mut u8
        }
    }

    let mut address_space = AddressSpace::new(InitContext { allocator }).unwrap_or_else(|_| oom());

    // Identity map the whole address space.
    address_space
        .map_range(0, 0, upper_bound as usize, PageTableFlags::WRITABLE)
        .unwrap_or_else(|err| handle_mapping_error(err));
    let page_directory = address_space.page_directory();
    address_space.leak();

    asm!(
        // Update the CR3 register with our page directory.
        "
        mov cr3, {page_directory}
        ",
        // Make sure that the PSE is enabled (this is necessary to use 4MiB mappings).
        "
        mov {tmp}, cr4
        or {tmp}, 0x00000010
        mov cr4, {tmp}
        ",
        // Enable paging.
        "
        mov {tmp}, cr0
        or {tmp}, 0x80000000
        mov cr0, {tmp}
        ",
        page_directory = in(reg) page_directory,
        tmp = lateout(reg) _,
    );
}

/// Handle a mapping error occuring within the initialization routine.
fn handle_mapping_error(err: MappingError) -> ! {
    match err {
        MappingError::OutOfMemory => oom(),
        MappingError::AlreadyMapped => panic!("attempted to map a region that was already mapped"),
    }
}

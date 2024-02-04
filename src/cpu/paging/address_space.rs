use crate::cpu::paging::PageTableIndex;
use crate::state::OutOfMemory;

use super::{PageTable, PageTableFlags};

/// The size of a single 4 KiB page.
const FOUR_KIB: usize = 4096;

/// The size of a single 4 MiB page.
const FOUR_MIB: usize = 4096 * 1024;

/// An error that might occur while mapping memory.
#[derive(Debug)]
pub enum MappingError {
    /// The system is out of memory and cannot allocate for more pages.
    OutOfMemory,
    /// The requested mapping conflicts with an existing one.
    AlreadyMapped,
}

impl From<OutOfMemory> for MappingError {
    #[inline(always)]
    fn from(_value: OutOfMemory) -> Self {
        Self::OutOfMemory
    }
}

/// Represent an address space.
pub struct AddressSpace<C> {
    /// The context used to manipulate the page table.
    context: C,
    /// The root page table.
    ///
    /// This is a physical address.
    root: u32,
}

impl<C: Context> AddressSpace<C> {
    /// Creates a new [`AddressSpace`] instance.
    pub fn new(mut context: C) -> Result<Self, OutOfMemory> {
        let root = context.allocate()?;

        unsafe {
            let root_ptr = context.map(root) as *mut PageTable;
            root_ptr.write_bytes(0x00, 1);
        }

        Ok(Self { context, root })
    }

    /// Returns the physical address of the page directory.
    #[inline(always)]
    pub fn page_directory(&self) -> u32 {
        self.root
    }

    /// Prevents the address space from being deallocated.
    #[inline]
    pub fn leak(self) {
        core::mem::forget(self);
    }

    /// Translates the provided virtual address to a physical address, if it is mapped.
    pub fn translate(&self, virt: usize) -> Option<u32> {
        let dir = unsafe { &*(self.context.map(self.root) as *const PageTable) };
        let pde = &dir[PageTableIndex::extract_page_directory_index(virt)];

        let pt = if !pde.is_present() {
            return None;
        } else if pde.is_huge_page() {
            return Some(PageTableIndex::extract_4mib_offset(virt) + pde.address_4mib());
        } else {
            // The page is present. We need to continue reading the page table.
            unsafe { &*(self.context.map(pde.address_4kib()) as *const PageTable) }
        };

        let pte = &pt[PageTableIndex::extract_page_table_index(virt)];

        if pte.is_present() {
            Some(PageTableIndex::extract_4kib_offset(virt) + pte.address_4kib())
        } else {
            None
        }
    }

    /// Maps a 4 KiB virtual page to a specific physical page.
    ///
    /// The flags of `entry` are properly dispatched to its parent entries.
    ///
    /// # Panics
    ///
    /// This function panics in debug builds if the provided virtual address
    /// is not properly aligned to a 4 KiB boundary.
    pub fn map_4kib(
        &mut self,
        virt: usize,
        phys: u32,
        flags: PageTableFlags,
    ) -> Result<(), MappingError> {
        debug_assert!(
            virt % FOUR_KIB == 0,
            "virtual address is not properly aligned to 4 KiB"
        );
        debug_assert!(
            phys as usize % FOUR_KIB == 0,
            "physical address is not properly aligned to 4 KiB"
        );
        debug_assert!(
            !flags.intersects(PageTableFlags::PRESENT | PageTableFlags::HUGE_PAGE),
            "invalid flags provided"
        );

        // Read the page directory.
        let dir = unsafe { &mut *(self.context.map(self.root) as *mut PageTable) };

        // Read the page directory entry.
        let pde_index = PageTableIndex::extract_page_directory_index(virt);
        let pde = &mut dir[pde_index];

        let pta = if !pde.is_present() {
            // The page directory entry is not present. We need to allocate
            // a page table for it.
            let pta = self.context.allocate()?;
            let pta_ptr;

            // Initialize the page table.
            unsafe {
                pta_ptr = self.context.map(pta) as *mut PageTable;
                pta_ptr.write_bytes(0x00, 1);
            }

            // Update the page directory entry.
            *pde = flags | PageTableFlags::PRESENT | PageTableFlags::from_bits_retain(phys);

            unsafe { &mut *pta_ptr }
        } else if pde.is_huge_page() {
            // The entry corresponds to a 4 MiB page. We cannot map a 4 KiB page here
            // as it is already mapped.
            return Err(MappingError::AlreadyMapped);
        } else {
            // The page directory entry is present. We need to update its
            // flags conservatively.
            update_flags(pde, flags);

            // And read it to get the page table address.
            let pta = pde.address_4kib();

            unsafe {
                let pta_ptr = self.context.map(pta) as *mut PageTable;
                &mut *pta_ptr
            }
        };

        let pte_index = PageTableIndex::extract_page_table_index(virt);
        let pte = &mut pta[pte_index];

        if pte.is_present() {
            // The page table entry is already present. We cannot map the page.
            return Err(MappingError::AlreadyMapped);
        } else {
            // Otherwise, we can map the page.
            *pte = flags | PageTableFlags::PRESENT | PageTableFlags::from_bits_retain(phys);
        }

        Ok(())
    }

    /// Maps a 4 MiB virtual page to a specific physical page.
    ///
    /// The flags of `entry` are properly dispatched to its parent entries.
    ///
    /// # Panics
    ///
    /// This function panics in debug builds if the provided virtual address
    /// is not properly aligned to a 4 MiB boundary.
    pub fn map_4mib(
        &mut self,
        virt: usize,
        phys: u32,
        flags: PageTableFlags,
    ) -> Result<(), MappingError> {
        debug_assert!(
            virt % FOUR_MIB == 0,
            "virtual address is not properly aligned to 4 MiB"
        );
        debug_assert!(
            phys as usize % FOUR_MIB == 0,
            "entry does not represent a 4 MiB page"
        );
        debug_assert!(
            !flags.intersects(PageTableFlags::PRESENT | PageTableFlags::HUGE_PAGE),
            "invalid flags provided"
        );

        // Read the page directory.
        let dir = unsafe { &mut *(self.context.map(self.root) as *mut PageTable) };

        // Read the page directory entry.
        let pde_index = PageTableIndex::extract_page_directory_index(virt);
        let pde = &mut dir[pde_index];

        if pde.is_present() {
            // The page directory entry is already mapped somewhere.
            return Err(MappingError::AlreadyMapped);
        }

        *pde = flags
            | PageTableFlags::PRESENT
            | PageTableFlags::HUGE_PAGE
            | PageTableFlags::from_bits_retain(phys);

        Ok(())
    }

    /// Maps a range of virtual pages to a range of physical pages.
    ///
    /// # Panics
    ///
    /// In debug builds, this function panics if any of the provided addresses or
    /// size are not properly aligned to a 4 KiB boundary.
    ///
    /// # Errors
    ///
    /// This function fails if any part of the mapping is already present in the
    /// virtual address space. Note that in that case, the function does not
    /// attempt to unmap the pages that were successfully mapped before the error
    /// occurred.
    pub fn map_range(
        &mut self,
        mut virt: usize,
        mut phys: u32,
        mut length: usize,
        flags: PageTableFlags,
    ) -> Result<(), MappingError> {
        debug_assert!(virt % FOUR_KIB == 0);
        debug_assert!(phys as usize % FOUR_KIB == 0);
        debug_assert!(length % FOUR_KIB == 0);

        while length != 0 {
            if length >= FOUR_MIB && virt % FOUR_MIB == 0 && phys as usize % FOUR_MIB == 0 {
                // We can map a 4 MiB page.
                self.map_4mib(virt, phys, flags)?;
                virt += FOUR_MIB;
                phys += FOUR_MIB as u32;
                length -= FOUR_MIB;
            } else {
                // We can only map a 4 KiB page.
                self.map_4kib(virt, phys, flags)?;
                virt += FOUR_KIB;
                phys += FOUR_KIB as u32;
                length -= FOUR_KIB;
            }
        }

        Ok(())
    }
}

fn update_flags(parent: &mut PageTableFlags, child: PageTableFlags) {
    // TODO: properly fuse the flags.
    *parent |= child;
}

/// Contains the functions required to manipulate a page table.
///
/// # Safety
///
/// Users of this trait will assume that it has been properly implemented, meaning that
/// the memory it gives out is logically owned by the acquirer, and that the mapping
/// remain valid.
///
/// If an address space change causes a mapping to be invalidated, it is the responsability
/// of the caller to update contexts accordingly.
pub unsafe trait Context {
    /// Allocate a new page physical page.
    ///
    /// # Returns
    ///
    /// This function returns the physical address of the allocated page.
    fn allocate(&mut self) -> Result<u32, OutOfMemory>;

    /// Deallocates the provided page.
    ///
    /// # Safety
    ///
    /// The provided `page` must have been allocated previously by the same [`Context`]
    /// instance.
    unsafe fn deallocate(&mut self, page: u32);

    /// Maps the provided physical address to a virtual address.
    ///
    /// # Safety
    ///
    /// The provided physical address must have been allocated by the same [`Context`] instance.
    /// Note that if paging is not yet enabled, this function should simply return the input
    /// address.
    unsafe fn map(&self, physical: u32) -> *mut u8;
}

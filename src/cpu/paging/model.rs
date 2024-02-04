use core::ops::{Index, IndexMut};

use bitflags::bitflags;

bitflags! {
    /// Represents the bits that a page table entry can have.
    #[derive(Debug, Clone, Copy)]
    pub struct PageTableFlags: u32 {
        /// Indicates that the entry is present.
        const PRESENT = 1 << 0;
        /// Whether the page can be written to. When set, the page is read/write. Otherwise, it
        /// is read-only.
        const WRITABLE = 1 << 1;
        /// Whether the page can be accessed by user mode code.
        const USER_ACCESSIBLE = 1 << 2;
        /// Whether the page has a write-through caching policy instead of using write-back
        /// caching.
        const WRITE_THROUGH = 1 << 3;
        /// Whether caching should be entirely disabled for the page.
        const CACHE_DISABLED = 1 << 4;
        /// This bit is set automatically by the CPU when it accesses the page. However, it
        /// does not attempt to clear it. This should be done by the OS.
        const ACCESSED = 1 << 5;
        /// Whether the page is dirty (i.e. it has been written to). Like `ACCESSED`, this bit
        /// is set automatically by the CPU and should be cleared by the OS when a transition
        /// needs to be observed.
        const DIRTY = 1 << 6;
        /// Only available on a page directory entry. When set, the entry references
        /// a huge (4 MiB) page, instead of referencing a page table.
        ///
        /// Note that in that case, the page must be aligned to 4 MiB.
        const HUGE_PAGE = 1 << 7;
        /// Only available on a page directory entry. When set, the page is global, meaning
        /// that it will be assumed not to change when the CR3 register is loaded with a new
        /// value.
        ///
        /// This means that the page directory entry is not flushed from the TLB when the CR3
        /// register is overwritten.
        const GLOBAL = 1 << 8;
    }
}

impl PageTableFlags {
    /// Returns whether the entry is present.
    #[inline(always)]
    pub fn is_present(&self) -> bool {
        self.intersects(Self::PRESENT)
    }

    /// Returns whether the entry is actually a HUGE_PAGE.
    ///
    /// Note that is is incorrect to call this function on a page table entry.
    #[inline(always)]
    pub fn is_huge_page(&self) -> bool {
        self.intersects(Self::HUGE_PAGE)
    }

    /// Returns the 4 KiB-aligned address of the page table or page directory (depending
    /// on the position).
    ///
    /// If you are reading from a page directory entry, it's probably incorrect to call
    /// this function when the HUGE_PAGE bit is set.
    #[inline(always)]
    pub fn address_4kib(&self) -> u32 {
        self.bits() & !0xFFF
    }
}

/// Represents a page table or page directory (depending on where it is located).
#[derive(Clone, Copy, Debug)]
#[repr(align(4096))]
pub struct PageTable([PageTableFlags; 1024]);

impl Index<PageTableIndex> for PageTable {
    type Output = PageTableFlags;

    #[inline(always)]
    fn index(&self, index: PageTableIndex) -> &Self::Output {
        unsafe { self.0.get_unchecked(index.as_usize()) }
    }
}

impl IndexMut<PageTableIndex> for PageTable {
    #[inline(always)]
    fn index_mut(&mut self, index: PageTableIndex) -> &mut Self::Output {
        unsafe { self.0.get_unchecked_mut(index.as_usize()) }
    }
}

/// An index within a [`PageTable`].
///
/// This index is used to access a specific entry within a page table without
/// having to check any bounds.
#[derive(Clone, Copy, Debug)]
pub struct PageTableIndex(u16);

impl PageTableIndex {
    /// Creates a new page table index from the provided value.
    pub fn new(value: usize) -> Self {
        debug_assert!(value < 1024, "index out of bounds");
        Self(value as u16)
    }

    /// Converts the index to a usize.
    #[inline(always)]
    pub fn as_usize(self) -> usize {
        self.0 as usize
    }

    /// Extracts the page directory index of the provided virtual address.
    #[inline]
    pub fn extract_page_directory_index(virt_addr: usize) -> Self {
        Self::new(virt_addr >> 22)
    }

    /// Extracts the page table index of the provided virtual address.
    #[inline]
    pub fn extract_page_table_index(virt_addr: usize) -> Self {
        Self::new((virt_addr >> 12) & 0x3FF)
    }

    /// Extracts the page offset of the provided virtual address.
    #[inline]
    pub fn extract_offset(virt_addr: usize) -> Self {
        Self::new(virt_addr & 0xFFF)
    }
}

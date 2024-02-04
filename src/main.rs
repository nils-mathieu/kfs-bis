#![no_std]
#![no_main]
#![feature(
    naked_functions,
    maybe_uninit_uninit_array,
    const_maybe_uninit_uninit_array,
    asm_const,
    decl_macro,
    abi_x86_interrupt,
    panic_info_message,
    pointer_is_aligned
)]
#![allow(dead_code)]

mod cpu;
mod die;
mod drivers;
mod multiboot;
mod shell;
mod state;
mod terminal;
mod utility;

use core::alloc::Layout;
use core::arch::asm;
use core::ffi::CStr;
use core::fmt::Write;
use core::mem::MaybeUninit;

use crate::shell::ReadLineImpl;

use self::cpu::paging::{AddressSpace, Context, MappingError, PageTableFlags};
use self::die::{die, oom};
use self::drivers::{pic, vga};
use self::multiboot::MultibootInfo;
use self::state::{Allocator, Global, SystemInfo};
use self::terminal::Terminal;
use self::utility::instr::{hlt, sti};
use self::utility::{HumanBytes, InitAllocator, Mutex};

/// The global terminal. It needs to be locked in order to be used.
static TERMINAL: Mutex<Terminal> = Mutex::new(Terminal::new(unsafe { vga::VgaBuffer::new() }));

/// Prints a message to the terminal.
pub macro printk($($args:tt)*) {{
	let _ = ::core::fmt::Write::write_fmt(
		$crate::TERMINAL.lock().as_mut(),
		::core::format_args!($($args)*)
	);
}}

/// Only used in the [`log!`] macro.
#[doc(hidden)]
fn __log(msg: core::fmt::Arguments) {
    #[cfg(feature = "log_serial")]
    crate::drivers::serial::__log(msg);
}

/// Logs a message.
pub macro log($($args:tt)*) {{
	$crate::__log(::core::format_args!($($args)*));
}}

/// The header that the bootloader will run to determine the features that the kernel wants.
#[link_section = ".multiboot_header"]
#[used]
static MULTIBOOT_HEADER: multiboot::Header =
    multiboot::Header::new(multiboot::HeaderFlags::MEMORY_MAP);

/// The size of the initial stack. See [`INIT_STACK`] for more information.
const INIT_STACK_SIZE: usize = 0x2000;
/// The initial stack used up until a proper allocator is available. It should not need to be too
/// large; just enough to get the kernel to a point where it can allocate physical memory
/// dynamically.
static mut INIT_STACK: [MaybeUninit<u8>; INIT_STACK_SIZE] = MaybeUninit::uninit_array();

/// This function is called by the bootloader.
///
/// It assumes that the protocol used is "multiboot" (first version, not multiboot2).
///
/// # Safety
///
/// This function expects to be called by a multiboot-compliant bootloader, meaning that the
/// current state of the machine must be compliant with the protocol.
#[no_mangle]
#[naked]
unsafe extern "C" fn entry_point() {
    asm!(
        // Check whether the multiboot magic number is valid.
        // When the value is not found, the CPU is left hanging.
        "
        cmp eax, {eax_magic}
        jne 2f
        ",
        // Setup the stack pointer.
        // The Grub bootloader actually provides a seemingly valid stack pointer, but it's
        // better to set it up ourselves to avoid relying on the bootloader for too long.
        "
        lea esp, [{init_stack_ptr} + {init_stack_size}]
        mov ebp, esp
        ",
        // Finally, call the Rust entry point for further initialization.
        // The bootloader has provided a pointer to the multiboot info structure in the `ebx`
        // register, which we pass as an argument to the other function.
        "
        push ebx
        call {entry_point2}
        ",
        // This is an infinite loop used to avoid resetting the CPU when the main function
        // returns, or when an error occurs during the initialization process.
        "
    2:
        hlt
        jmp 2b
        ",
        eax_magic = const multiboot::EAX_MAGIC,
        init_stack_ptr = sym INIT_STACK,
        init_stack_size = const INIT_STACK_SIZE,
        entry_point2 = sym entry_point2,
        options(noreturn)
    )
}

/// The second entry point function of the kernel, called within [`entry_point`].
///
/// # Safety
///
/// This function may only be called once by the `entry_point` function defined above.
unsafe extern "C" fn entry_point2(info: &MultibootInfo) {
    // Initialize the terminal and set up the cursor. Doing this now avoid as much as possible
    // screen flickering while the kernel is initializing.
    log!("Initializing the terminal...\n");
    vga::cursor_show(15, 15);
    TERMINAL.lock().reset();

    // Print information about the bootloader.
    if info.flags.intersects(multiboot::InfoFlags::BOOTLOADER_NAME) {
        let name = CStr::from_ptr(info.bootloader_name);
        log!("Bootloader: {:?}\n", name);
    } else {
        log!("Bootloader has not provided its name.\n");
    }

    // Initialize the CPU and other hardware components.
    log!("Initializing the CPU...\n");
    cpu::gdt::init();
    cpu::idt::init();
    pic::init();
    pic::set_irq_mask(!pic::Irqs::KEYBOARD);

    // Read the memory map.
    log!("Reading the memory map...\n");
    if !info.flags.intersects(multiboot::InfoFlags::MEMORY_MAP) {
        TERMINAL.lock().set_color(vga::Color::Red);
        die("the bootloader did not provid a memory map");
    }
    let total_memory = multiboot::iter_memory_map(info.mmap_addr, info.mmap_length)
        .filter(|e| e.ty == multiboot::MemMapType::AVAILABLE)
        .map(|e| e.len_low as u64 | (e.len_high as u64) << 32)
        .sum::<u64>();
    log!("Found {} of available memory.\n", HumanBytes(total_memory));

    // Find the largest available segment. Avoid memory under one megabyte as that's
    // were a lot of available (but used!) memory is located. For example the VGA
    // memory is down there.
    let largest_segment = multiboot::iter_memory_map(info.mmap_addr, info.mmap_length)
        .filter(|e| e.ty == multiboot::MemMapType::AVAILABLE)
        .map(|e| {
            (
                e.addr_low as u64 | (e.addr_high as u64) << 32,
                e.len_low as u64 | (e.len_high as u64) << 32,
            )
        })
        .filter(|&(addr, _)| addr >= 0x100000)
        .max_by_key(|&(_, len)| len)
        .unwrap_or_else(|| {
            die("no available memory segment found above 1MB");
        });
    log!(
        "Largest memory segment: {:#016x} -> {:#016x} ({})\n",
        largest_segment.0,
        largest_segment.0 + largest_segment.1,
        HumanBytes(largest_segment.1)
    );

    // Make sure that the bounds of the largest segment are within the supported address
    // space. This should generally be the case, but it's better to be safe than sorry.
    if largest_segment.0 > u32::MAX as u64
        || largest_segment.0.saturating_add(largest_segment.1) > u32::MAX as u64
    {
        die("the largest memory segment is not within the supported address space (4 GiB)");
    }

    // Create the boot allocator that will be used to set up everything else.
    let mut init_allocator = unsafe {
        InitAllocator::new(
            largest_segment.0 as usize,
            (largest_segment.0 + largest_segment.1) as usize,
        )
    };

    log!("Setting up the kernel's address-space (identity mapping)\n");
    struct InitContext<'a> {
        init_allocator: &'a mut InitAllocator,
    }

    unsafe impl<'a> Context for InitContext<'a> {
        #[inline]
        fn allocate(&mut self) -> Result<u32, state::OutOfMemory> {
            let layout = unsafe { Layout::from_size_align_unchecked(4096, 4096) };
            self.init_allocator
                .try_allocate_raw(layout)
                .map(|addr| addr as u32)
        }

        unsafe fn deallocate(&mut self, _: u32) {
            unreachable!("this Context implementation should never be used to deallocate pages");
        }

        #[inline]
        unsafe fn map(&mut self, physical: u32) -> *mut u8 {
            // At this point in the execution, we are setting up the kernel's address space, meaning
            // that paging is not yet initiating. Every "virtual" address is equal to its
            // physical address.
            physical as *mut u8
        }
    }

    let mut address_space = AddressSpace::new(InitContext {
        init_allocator: &mut init_allocator,
    })
    .unwrap_or_else(|_| oom());

    // Identity map the whole address space.
    let upper_bound = (largest_segment.0 + largest_segment.1) as usize;
    address_space
        .map_range(0, 0, upper_bound & !0xFFF, PageTableFlags::WRITABLE)
        .unwrap_or_else(|err| handle_mapping_error(err));
    let page_directory = address_space.page_directory();
    address_space.leak();

    log!("Initializing the physical memory allocator...\n");
    // Go through the available segments and compute the total amount of memory
    // that needs to be tracked.
    let iter = multiboot::iter_memory_map(info.mmap_addr, info.mmap_length)
        .filter(|e| e.ty == multiboot::MemMapType::AVAILABLE)
        .map(|e| {
            (
                e.addr_low as u64 | (e.addr_high as u64) << 32,
                e.len_low as u64 | (e.len_high as u64) << 32,
            )
        })
        .filter(|&(addr, _)| addr >= 0x100000 && addr <= u32::MAX as u64 - 0x1000)
        .map(|(addr, len)| {
            (
                addr as u32,
                addr.checked_add(len)
                    .unwrap_or(u32::MAX as u64)
                    .min(u32::MAX as u64) as u32,
            )
        })
        .map(|(start, end)| ((start + 0xFFF) & !0xFFF, end & !0xFFF))
        .flat_map(|(start, end)| (start..=end).step_by(0x1000));
    let allocator_storage = init_allocator.allocate_slice(iter.clone().count());
    log!(
        "The allocator can track up to {} physical pages.\n",
        allocator_storage.len()
    );
    let mut allocator = Allocator::new(allocator_storage);

    for page in iter {
        debug_assert!(page % 0x1000 == 0);
        allocator.deallocate(page);
    }

    let used_memory = (largest_segment.0 + largest_segment.1) as usize - init_allocator.top();
    let remaining_memory = total_memory - used_memory as u64;
    log!(
        "Finished utilizing the boot allocator (used: {}, remaining: {})\n",
        HumanBytes(used_memory as u64),
        HumanBytes(remaining_memory)
    );

    log!("Switching address space...\n");
    asm!(
        "mov cr3, {}",
        in(reg) page_directory,
    );

    // Write the global state.
    log!("Initilizing the global state...\n");
    crate::state::GLOBAL
        .set(Global {
            system_info: SystemInfo { total_memory },
            allocator: Mutex::new(allocator),
        })
        .ok()
        .expect("global state already initialized");

    // Enable interrupts.
    log!("Enabling interrupts...\n");
    sti();

    let _ = TERMINAL.lock().write_str(include_str!("welcome.txt"));

    loop {
        hlt();
        TERMINAL.lock().take_buffered_scancodes(&mut ReadLineImpl);
    }
}

/// Handle a mapping error occuring within the initialization routine.
fn handle_mapping_error(err: MappingError) -> ! {
    match err {
        MappingError::OutOfMemory => oom(),
        MappingError::AlreadyMapped => panic!("attempted to map a region that was already mapped"),
    }
}

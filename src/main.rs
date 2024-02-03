#![no_std]
#![no_main]
#![feature(naked_functions)]
#![feature(maybe_uninit_uninit_array, const_maybe_uninit_uninit_array)]
#![feature(asm_const)]
#![feature(decl_macro)]
#![feature(abi_x86_interrupt)]
#![allow(dead_code)]

mod cpu;
mod drivers;
mod multiboot;
mod sync;
mod terminal;
mod utility;
mod vga;

use core::arch::asm;
use core::mem::MaybeUninit;
use core::panic::PanicInfo;

use crate::drivers::{pic, ps2};
use crate::utility::instr::sti;

use self::sync::Mutex;
use self::terminal::Terminal;
use self::utility::instr::{cli, hlt};
use self::vga::VgaBuffer;

/// The global terminal. It needs to be locked in order to be used.
static TERMINAL: Mutex<Terminal> = Mutex::new(Terminal::new(unsafe { VgaBuffer::new() }));

/// Prints a message to the terminal.
pub macro printk($($args:tt)*) {{
	let _ = ::core::fmt::Write::write_fmt(
		$crate::TERMINAL.lock().as_mut(),
		::core::format_args!($($args)*)
	);
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
unsafe extern "C" fn entry_point2(_info: u32) {
    // Initialize the terminal and set up the cursor.
    vga::cursor_show(15, 15);
    TERMINAL.lock().reset();

    // Initialize the CPU.
    cpu::gdt::init();
    cpu::idt::init();
    pic::init();
    pic::set_irq_mask(!pic::Irqs::KEYBOARD);
    sti();

    printk!("42\n");

    loop {
        hlt();
        TERMINAL.lock().take_buffered_scancodes();
    }
}

/// This function is called when something in the kernel panics.
///
/// If the control flow of the kernel ever reaches this point, it means that something
/// went terribly wrong and the kernel may be in an inconsistent state.
#[panic_handler]
fn die_and_catch_fire(info: &PanicInfo) -> ! {
    TERMINAL.lock().set_color(vga::Color::Red);
    printk!("{info}");

    cli();
    loop {
        hlt();
    }
}

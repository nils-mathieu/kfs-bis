//! This module defines various error functions that are used throughout the kernel.

use core::fmt::Write;
use core::panic::PanicInfo;

use crate::drivers::{ps2, vga};
use crate::utility::instr::{cli, hlt, outb, pause};
use crate::{log, TERMINAL};

/// Kills the kernel with an appropriate message indicating that the system has run
/// out of memory.
///
/// This function should only be called during initialization, as it is not really
/// possible to recover from lacking memory.
///
/// However, once the kernel is running normally, memory errors should be handled
/// properly and recovered from.
#[inline]
pub fn oom() -> ! {
    die("please download more RAM");
}

/// Restarts the CPU.
pub fn reset_cpu() -> ! {
    // This is probably just triggering a tripple fault. The documentation online does not
    // seem to agree on what this does exactly. The proper way to do this would be to
    // use the ACPI, but it's a bit out of scope.
    unsafe { outb(0xCF9, 0xE) };

    loop {
        hlt();
    }
}

/// This function is called when something in the kernel panics.
///
/// If the control flow of the kernel ever reaches this point, it means that something
/// went terribly wrong and the kernel may be in an inconsistent state.
#[panic_handler]
#[cold]
#[inline(never)]
fn die_and_catch_fire(info: &PanicInfo) -> ! {
    cli();

    // SAFETY:
    //  We just made sure that no interrupts can occur, meaning that this mutable reference
    //  at most overlaps with the current thread (if the lock was helf while the panic
    //  occured). In that case, this operation is technically unsound. This should be fine,
    //  however, as the kernel is about to die anyway. The chances that the compiler is able
    //  to optimize this in a harmful way are slim.
    let term = unsafe { TERMINAL.get_mut_unchecked() };

    vga::cursor_hide();
    term.set_color(vga::Color::Red);
    term.clear_cmdline();

    // Write a message explaining what happened:
    log!("\n\nKERNEL PANIC:\n{}", info);

    let _ = writeln!(
        term,
        "\
      	The kernel panicked unexpectedly. This is a serious bug in the operating\n\
        system. Press any key in order to restart the computer.\n\
        \n\
        Additional information:\
        "
    );

    if let Some(location) = info.location() {
        let _ = writeln!(term, "> LOCATION: {}", location);
    }

    if let Some(msg) = info.message() {
        let _ = writeln!(term, "> MESSAGE:\n{}", msg);
    }

    wait_any_key();
    reset_cpu();
}

/// Function called when something in the kernel goes wrong, but without it being
/// a bug.
///
/// For example, if the kernel cannot initialize itself because of a lack of working
/// memory, this function will be called.
///
/// # Panics
///
/// This function panics if the terminal is currently locked.
#[cold]
pub fn die(error: &str) -> ! {
    cli();

    {
        let mut term = TERMINAL.lock();
        term.set_color(vga::Color::Red);
        term.clear_cmdline();
        let _ = writeln!(
            term,
            "\nFATAL ERROR: {error}\n\nPress any key to restart the computer...\n",
        );

        log!("FATAL ERROR: {error}");
    }

    wait_any_key();
    reset_cpu();
}

/// Blocks the execution of the current thread until the user presses any key.
///
/// # Notes
///
/// This function expects interrupts to be disabled, and that no other part of the
/// code is accessing the PS2 output buffer. Failing to meet those conditions might
/// prevent the function from ever returning.
fn wait_any_key() {
    loop {
        while !ps2::is_output_buffer_full() {
            pause();
        }

        // If the most significant bit is set, then the scancode is a MAKE code
        // instead of a BREAK code. This avoid continuing unintentionally when
        // the user releases a key.
        if ps2::read_data() & 0x80 == 0 {
            break;
        }
    }
}

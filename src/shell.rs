//! Provides a simple shell implementation.

use core::fmt::Write;

use crate::die::reset_cpu;
use crate::drivers::vga;
use crate::state::GLOBAL;
use crate::terminal::{ReadLine, Terminal};
use crate::utility::HumanBytes;

/// A simple implementation of the `ReadLine` trait for the terminal.
pub struct ReadLineImpl;

/// The list of available commands.
#[allow(clippy::type_complexity)]
const COMMANDS: &[(&[u8], fn(&mut Terminal))] = &[
    (b"help", help),
    (b"clear", clear),
    (b"font", font),
    (b"system", system),
    (b"panic", panic),
    (b"restart", restart),
];

impl ReadLine for ReadLineImpl {
    fn submit(&mut self, term: &mut Terminal) {
        for &(cmd, handler) in COMMANDS {
            if term.cmdline() == cmd {
                handler(term);
                return;
            }
        }
    }

    fn auto_complete(&mut self, term: &mut Terminal) {
        if term.cmdline().is_empty() || term.cmdline_cursor() != term.cmdline().len() {
            return;
        }

        for (cmd, _) in COMMANDS {
            if cmd.starts_with(term.cmdline()) {
                term.cmdline_mut().clear();
                term.cmdline_mut().extend_from_slice(cmd);
                term.set_cmdline_cursor(term.cmdline().len());
                term.refresh_cmdline();
            }
        }
    }
}

/// The `help` command.
pub fn help(term: &mut Terminal) {
    term.insert_linefeed();
    let _ = term.write_str(include_str!("help.txt"));
}

/// The `clear` command.
pub fn clear(term: &mut Terminal) {
    term.reset();
}

/// The `font` command.
pub fn font(term: &mut Terminal) {
    let _ = term.write_str("\nAvailable characters:\n");
    for i in vga::VgaChar::iter_all() {
        term.write_vga_char(i);
    }

    let _ = term.write_str("\n\nAvailable colors:\n");
    for c in vga::Color::iter_all() {
        term.set_color(c);
        term.write_vga_char(vga::VgaChar::BLOCK);
        term.write_vga_char(vga::VgaChar::BLOCK);
    }
    term.set_color(vga::Color::White);
    term.insert_linefeed();
}

/// The `system` command.
pub fn system(term: &mut Terminal) {
    let glob = GLOBAL.get().unwrap();

    let total_memory = glob.system_info.total_memory;
    let remaining_memory = glob.allocator.lock().remaining_memory() as u64;
    let bootloader_name = glob
        .system_info
        .bootloader_name
        .as_ref()
        .map(|x| core::str::from_utf8(x).unwrap_or("<invalid utf-8>"))
        .unwrap_or("<unknown>");

    let _ = writeln!(
        term,
        "\n\
        bootloader: {bootloader_name}
        \n\
      	total memory: {memory} ({memory_b} bytes)\n\
        remaining memory: {remaining} ({remaining_b} bytes)\n\
       	",
        memory = HumanBytes(total_memory as u64),
        memory_b = total_memory,
        remaining = HumanBytes(remaining_memory),
        remaining_b = remaining_memory,
    );
}

/// The `panic` command.
pub fn panic(_term: &mut Terminal) {
    panic!("why would they add this command in the first place???");
}

/// The `restart` command.
pub fn restart(_term: &mut Terminal) {
    reset_cpu();
}

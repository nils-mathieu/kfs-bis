//! This module provides a simple terminal implementation backed by the VGA buffer.

mod layouts;

use core::fmt::Write;

use crate::drivers::vga::{self, Color, VgaBuffer, VgaChar, HEIGHT, WIDTH};
use crate::utility::ArrayVec;

/// Contains the state of the terminal.
pub struct Terminal {
    /// The underlying buffer on which we are writing.
    screen: VgaBuffer,

    /// The current position of the cursor (the column to which the next character
    /// will be written).
    ///
    /// This is always between 0 and 80 (included). When this value is equal to 80,
    /// a new line is started and the cursor is reset to 0.
    cursor: u32,

    /// The current foreground color.
    foreground: Color,

    /// The current command line.
    cmdline: ArrayVec<u8, { WIDTH as usize }>,
    /// The position of the user's cursor within the command-line.
    cmdline_cursor: u8,

    /// A bunch of scan-codes that have been received from the keyboard.
    ///
    /// This is a bounded queue.
    scancode_buffer: ArrayVec<u8, 8>,

    layout: layouts::Qwerty,
}

impl Terminal {
    /// Creates a new [`Terminal`] instance.
    pub const fn new(screen: VgaBuffer) -> Self {
        Self {
            screen,
            cursor: 0,
            foreground: Color::White,

            cmdline: ArrayVec::new(),
            cmdline_cursor: 0,

            scancode_buffer: ArrayVec::new(),

            layout: layouts::Qwerty::new(),
        }
    }

    /// Re-initializes the terminal.
    pub fn reset(&mut self) {
        self.cmdline.clear();
        self.cursor = 0;
        self.screen.buffer_mut().fill(CLEAR_VALUE);
        vga::cursor_move(0, HEIGHT - 1);
    }

    pub fn clear_cmdline(&mut self) {
        self.cmdline.clear();
        self.cmdline_cursor = 0;

        let w = WIDTH as usize;
        let h = HEIGHT as usize;
        self.screen.buffer_mut()[w * (h - 1)..].fill(CLEAR_VALUE);

        vga::cursor_move(0, HEIGHT - 1);
    }

    /// Scrolls the content of the terminal up by one line.
    pub fn scroll_once(&mut self) {
        let w = WIDTH as usize;
        let h = HEIGHT as usize;

        self.screen.buffer_mut().copy_within(w..w * (h - 1), 0);
        self.screen.buffer_mut()[w * (h - 2)..w * (h - 1)].fill(CLEAR_VALUE);
    }

    /// Inserts a line feed.
    ///
    /// This function does not necessarily scroll the terminal immediately. It only
    /// buffers the new line once for the next time a character is written.
    pub fn insert_linefeed(&mut self) {
        if self.cursor == WIDTH {
            self.scroll_once();
        }

        self.cursor = WIDTH;
    }

    /// Writes a character to the terminal.
    pub fn write_vga_char(&mut self, c: VgaChar) {
        if self.cursor == WIDTH {
            self.cursor = 0;
            self.scroll_once();
        }

        self.screen
            .putc(c, self.cursor, HEIGHT - 2, self.foreground, Color::Black);

        self.cursor += 1;
    }

    /// Sets the foreground color of the terminal.
    ///
    /// This only affects subsequent characters written to the terminal.
    #[inline(always)]
    pub fn set_color(&mut self, color: Color) {
        self.foreground = color;
    }

    /// Refreshes the written content of the command-line.
    ///
    /// This function should be called whenever the command-line is modified.
    pub fn refresh_cmdline(&mut self) {
        for (x, &c) in self.cmdline.iter().enumerate() {
            self.screen.putc(
                VgaChar::from_char(c as char)
                    .expect("found an invalid VGA character in the command line"),
                x as u32,
                HEIGHT - 1,
                Color::White,
                Color::Black,
            );
        }
        let w = WIDTH as usize;
        let h = HEIGHT as usize;
        let len = self.cmdline.len();
        self.screen.buffer_mut()[w * (h - 1) + len..].fill(CLEAR_VALUE);
        vga::cursor_move(self.cmdline_cursor as u32, HEIGHT - 1);
    }

    /// Inserts a new character into the command-line.
    ///
    /// # Returns
    ///
    /// This function returns whether the character could be inserted into the command-line.
    pub fn type_in(&mut self, c: u8) -> bool {
        if self
            .cmdline
            .try_insert(self.cmdline_cursor as usize, c)
            .is_err()
        {
            return false;
        }

        self.cmdline_cursor += 1;
        self.refresh_cmdline();

        true
    }

    /// Removes the characters currently under the cursor.
    ///
    /// When `bulk` is set, a whole word is removed.
    pub fn type_out(&mut self, bulk: bool) {
        let cur = self.cmdline_cursor as usize;

        if cur == 0 {
            return;
        }

        let start = if bulk {
            find_start_of_last_word(&self.cmdline[..cur])
        } else {
            cur - 1
        };

        self.cmdline.remove_range(start..cur);
        self.cmdline_cursor -= (cur - start) as u8;

        self.refresh_cmdline();
    }

    /// Caches the provided scan-code for later processing.
    ///
    /// This function is meant to be used within the interrupt handler and is very cheap
    /// to call.
    ///
    /// # Returns
    ///
    /// This function returns whether the scan-code could be taken into account. Specifically,
    /// it fails when the internal buffer is full.
    #[must_use = "the function might've failed to take the scan-code"]
    pub fn buffer_scancode(&mut self, scancode: u8) -> bool {
        self.scancode_buffer.try_push(scancode).is_ok()
    }

    /// Takes a scan-code and processes it.
    ///
    /// This function ignores the internal buffer and processes the scan-code immediately.
    pub fn take_scancode(&mut self, scancode: u8, readline: &mut dyn ReadLine) {
        let Some(c) = self.layout.advance(scancode) else {
            return;
        };

        // Process special characters. Those are used to control the terminal itself.
        match c {
            '\x08' => self.type_out(self.layout.modifiers().has_control()),
            'l' | 'L' if self.layout.modifiers().has_control() => self.reset(),
            'c' | 'C' if self.layout.modifiers().has_control() => self.clear_cmdline(),
            '\n' => {
                readline.submit(self);
                self.clear_cmdline();
            }
            '\t' => readline.auto_complete(self),
            _ => {
                self.type_in(c as u8);
            }
        }
    }

    /// Processes the scan-codes that were buffered so far.
    pub fn take_buffered_scancodes(&mut self, readline: &mut dyn ReadLine) {
        for i in 0..self.scancode_buffer.len() {
            let scancode = unsafe { *self.scancode_buffer.get_unchecked(i) };
            self.take_scancode(scancode, readline);
        }
        self.scancode_buffer.clear();
    }

    /// Returns an exclusive reference to the command-line buffer.
    #[inline(always)]
    pub fn cmdline_mut(&mut self) -> &mut ArrayVec<u8, { WIDTH as usize }> {
        &mut self.cmdline
    }

    /// Returns a shared reference to the command-line buffer.
    #[inline(always)]
    pub fn cmdline(&self) -> &[u8] {
        &self.cmdline
    }
}

impl Write for Terminal {
    fn write_char(&mut self, c: char) -> core::fmt::Result {
        if c == '\n' {
            self.insert_linefeed();
            return Ok(());
        }

        let c = VgaChar::from_char(c).ok_or(core::fmt::Error)?;
        self.write_vga_char(c);
        Ok(())
    }

    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        s.chars().try_for_each(|c| self.write_char(c))?;
        Ok(())
    }
}

/// The value used when clearing the terminal.
const CLEAR_VALUE: u16 = 0x0F00;

/// Returns the index of the first character of the last word.
///
/// If no word is found, 0 is returned.
fn find_start_of_last_word(s: &[u8]) -> usize {
    let mut i = s.len();

    // Skip initial whitespaces.
    while i > 0 {
        i -= 1;
        if s[i] != b' ' {
            break;
        }
    }

    // Skip the last word.
    while i > 0 {
        i -= 1;
        if s[i] == b' ' {
            return i + 1;
        }
    }

    i
}

/// Allows to customize the behavior of the terminal.
#[allow(unused_variables)]
pub trait ReadLine {
    /// Called when the user submits the current command-line value.
    fn submit(&mut self, term: &mut Terminal) {}

    /// Called when the user requests help for the current command-line value.
    fn auto_complete(&mut self, term: &mut Terminal) {}
}

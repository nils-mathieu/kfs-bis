//! This module provides a simple terminal implementation backed by the VGA buffer.

mod layouts;

use core::fmt::Write;

use crate::vga::{self, Color, VgaBuffer, VgaChar, HEIGHT, WIDTH};

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
    cmdline: [VgaChar; WIDTH as usize],
    /// The current length of the command line.
    cmdline_len: u8,
    /// The position of the user's cursor within the command-line.
    cmdline_cursor: u8,

    /// A bunch of scan-codes that have been received from the keyboard.
    ///
    /// This is a bounded queue.
    scancode_buffer: [u8; 8],
    /// The number of scan-codes currently in the buffer.
    scancode_buffer_len: u8,

    layout: layouts::Qwerty,
}

impl Terminal {
    /// Creates a new [`Terminal`] instance.
    pub const fn new(screen: VgaBuffer) -> Self {
        Self {
            screen,
            cursor: 0,
            foreground: Color::White,

            cmdline: [VgaChar::SPACE; WIDTH as usize],
            cmdline_len: 0,
            cmdline_cursor: 0,

            scancode_buffer: [0; 8],
            scancode_buffer_len: 0,

            layout: layouts::Qwerty::new(),
        }
    }

    /// Re-initializes the terminal.
    pub fn reset(&mut self) {
        self.cmdline_len = 0;
        self.cursor = 0;
        self.screen.buffer_mut().fill(CLEAR_VALUE);
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
        for i in 0..self.cmdline_len {
            self.screen.putc(
                self.cmdline[i as usize],
                i as u32,
                HEIGHT - 1,
                Color::White,
                Color::Black,
            );
        }
        let w = WIDTH as usize;
        let h = HEIGHT as usize;
        let len = self.cmdline_len as usize;
        self.screen.buffer_mut()[w * (h - 1) + len..].fill(CLEAR_VALUE);
        vga::cursor_move(self.cmdline_cursor as u32, HEIGHT - 1);
    }

    /// Inserts a new character into the command-line.
    ///
    /// # Returns
    ///
    /// This function returns whether the character could be inserted into the command-line.
    pub fn type_in(&mut self, c: VgaChar) -> bool {
        if self.cmdline_len == self.cmdline.len() as u8 {
            return false;
        }

        let cur = self.cmdline_cursor as usize;
        let len = self.cmdline_len as usize;
        self.cmdline.copy_within(cur..len, cur + 1);
        self.cmdline[cur] = c;

        self.cmdline_cursor += 1;
        self.cmdline_len += 1;

        self.refresh_cmdline();

        true
    }

    /// Removes the characters currently under the cursor.
    ///
    /// When `bulk` is set, a whole word is removed.
    pub fn type_out(&mut self, bulk: bool) {
        let cur = self.cmdline_cursor as usize;
        let len = self.cmdline_len as usize;

        if cur == 0 {
            return;
        }

        let count = if bulk {
            cur - find_start_of_last_word(&self.cmdline[..cur])
        } else {
            1
        };

        self.cmdline.copy_within(cur..len, cur - count);
        self.cmdline_cursor -= count as u8;
        self.cmdline_len -= count as u8;

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
        match self
            .scancode_buffer
            .get_mut(self.scancode_buffer_len as usize)
        {
            Some(slot) => {
                *slot = scancode;
                self.scancode_buffer_len += 1;
                true
            }
            None => false,
        }
    }

    /// Takes a scan-code and processes it.
    ///
    /// This function ignores the internal buffer and processes the scan-code immediately.
    pub fn take_scancode(&mut self, scancode: u8) {
        let Some(c) = self.layout.advance(scancode) else {
            return;
        };

        // Process special characters. Those are used to control the terminal itself.
        match c {
            '\x08' => self.type_out(self.layout.modifiers().has_control()),
            'l' | 'L' if self.layout.modifiers().has_control() => self.reset(),
            _ => {
                if let Some(vga_char) = VgaChar::from_char(c) {
                    self.type_in(vga_char);
                }
            }
        }
    }

    /// Processes the scan-codes that were buffered so far.
    pub fn take_buffered_scancodes(&mut self) {
        for i in 0..self.scancode_buffer_len {
            self.take_scancode(self.scancode_buffer[i as usize]);
        }
        self.scancode_buffer_len = 0;
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
fn find_start_of_last_word(s: &[VgaChar]) -> usize {
    let mut i = s.len();

    // Skip initial whitespaces.
    while i > 0 {
        i -= 1;
        if s[i] != VgaChar::SPACE {
            break;
        }
    }

    // Skip the last word.
    while i > 0 {
        i -= 1;
        if s[i] == VgaChar::SPACE {
            return i + 1;
        }
    }

    i
}

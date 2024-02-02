//! This module provides a simple terminal implementation backed by the VGA buffer.

use crate::vga::{Color, VgaBuffer, HEIGHT, WIDTH};

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
    cmdline: [u8; WIDTH as usize],
    /// The current length of the command line.
    cmdline_len: u32,
    /// The position of the user's cursor within the command-line.
    cmdline_cursor: u32,
}

impl Terminal {
    /// Creates a new [`Terminal`] instance.
    pub const fn new(screen: VgaBuffer) -> Self {
        Self {
            screen,
            cursor: 0,
            foreground: Color::White,
            cmdline: [0; WIDTH as usize],
            cmdline_len: 0,
            cmdline_cursor: 0,
        }
    }

    /// Re-initializes the terminal.
    pub fn reset(&mut self) {
        self.cmdline_len = 0;
        self.cursor = 0;

        self.clear();
        self.refresh_cmdline();
    }

    /// Clears the terminal to fully black.
    ///
    /// # Notes
    ///
    /// This function does not include the command-line area.
    pub fn clear(&mut self) {
        self.screen.buffer_mut()[..WIDTH as usize * (HEIGHT - 1) as usize].fill(CLEAR_VALUE);
    }

    /// Scrolls the content of the terminal up by one line.
    pub fn scroll_once(&mut self) {
        let w = WIDTH as usize;
        let h = HEIGHT as usize;

        self.screen.buffer_mut().copy_within(w..w * (h - 1), 0);
        self.screen.buffer_mut()[w * (h - 2)..w * (h - 1)].fill(CLEAR_VALUE);
    }

    /// Writes a character to the terminal.
    pub fn write_char(&mut self, c: u8) {
        if self.cursor == WIDTH {
            self.cursor = 0;
            self.scroll_once();
        }

        if c == b'\n' {
            self.cursor = WIDTH;
            return;
        }

        self.screen
            .putc(c, self.cursor, HEIGHT - 2, self.foreground, Color::Black);

        self.cursor += 1;
    }

    /// Writes the bytes of the provided string to the terminal.
    pub fn write_bytes(&mut self, s: &[u8]) {
        s.iter().for_each(|&c| self.write_char(c));
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
                i,
                HEIGHT - 1,
                Color::White,
                Color::Black,
            );
        }
        self.screen.buffer_mut()[WIDTH as usize * (HEIGHT - 1) as usize..].fill(CLEAR_VALUE);
        crate::vga::cursor_move(self.cmdline_cursor, HEIGHT - 1);
    }
}

impl core::fmt::Write for Terminal {
    fn write_char(&mut self, c: char) -> core::fmt::Result {
        let mut buf = [0; 4];
        self.write_bytes(c.encode_utf8(&mut buf).as_bytes());
        Ok(())
    }

    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_bytes(s.as_bytes());
        Ok(())
    }
}

/// The value used when clearing the terminal.
const CLEAR_VALUE: u16 = 0x0F00;

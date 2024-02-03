//! This module provides a simple VGA driver for writing characters to the screen.

use core::num::NonZeroU8;

use crate::utility::instr::{inb, outb};

/// Represents the VGA buffer.
///
/// Only one instance of this struct should exist at any given time.
pub struct VgaBuffer(());

impl VgaBuffer {
    /// Creates a new [`VgaBuffer`] instance.
    ///
    /// # Safety
    ///
    /// Only one [`VgaBuffer`] instance must exist at any given time.
    pub const unsafe fn new() -> Self {
        Self(())
    }

    /// Writes a character to the VGA buffer.
    ///
    /// # Safety
    ///
    /// The provided coordinates (x and y) must be within the bounds of the VGA buffer.
    ///
    /// Specifically `x` must be less than `WIDTH` and `y` must be less than `HEIGHT`.
    #[inline]
    pub unsafe fn putc_unchecked(&mut self, c: VgaChar, x: u32, y: u32, fg: Color, bg: Color) {
        let offset = y * WIDTH + x;
        let value = (c.as_u8() as u16) | ((bg as u16) << 12) | ((fg as u16) << 8);

        unsafe {
            *ADDRESS.add(offset as usize) = value;
        }
    }

    /// Writes a character to the VGA buffer.
    ///
    /// # Errors
    ///
    /// This function fails silently if the provided coordinates are out of bounds.
    #[inline]
    pub fn putc(&mut self, c: VgaChar, x: u32, y: u32, fg: Color, bg: Color) {
        if x < WIDTH && y < HEIGHT {
            unsafe {
                self.putc_unchecked(c, x, y, fg, bg);
            }
        }
    }

    /// Returns a shared slice reference over the underlying buffer.
    #[inline(always)]
    pub fn buffer(&self) -> &[u16] {
        unsafe { core::slice::from_raw_parts(ADDRESS, (WIDTH * HEIGHT) as usize) }
    }

    /// Returns an exclusive slice reference over the underlying buffer.
    #[inline(always)]
    pub fn buffer_mut(&mut self) -> &mut [u16] {
        unsafe { core::slice::from_raw_parts_mut(ADDRESS, (WIDTH * HEIGHT) as usize) }
    }
}

/// The width of the VGA buffer.
pub const WIDTH: u32 = 80;
/// The height of the VGA buffer.
pub const HEIGHT: u32 = 25;

/// The address of the VGA buffer.
const ADDRESS: *mut u16 = 0xB8000 as *mut u16;

/// A color supported by the VGA buffer.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

/// Updates the appearance of the cursor.
///
/// This function allows selecting a start and end scanline for the cursor.
///
/// # Panics
///
/// This function panics if any of the provided scanline numbers are out of
/// bounds. Both the start and end scanlines must be less than 16 (excluded).
pub fn cursor_show(start: u8, end: u8) {
    // Ensure that the provided bounds are valid.
    assert!(start < 16, "start scanline must be less than 16");
    assert!(end < 16, "end scanline must be less than 16");

    // Set the start and end scanlines for the cursor.
    unsafe {
        outb(0x3D4, 0x0A);
        outb(0x3D5, (inb(0x3D5) & 0xC0) | start);

        outb(0x3D4, 0x0B);
        outb(0x3D5, (inb(0x3D5) & 0xE0) | end);
    }
}

/// Hides the cursor.
pub fn cursor_hide() {
    unsafe {
        outb(0x3D4, 0x0A);
        outb(0x3D5, 0x20);
    }
}

/// Moves the cursor at the specified position.
///
/// # Errors
///
/// This function will fail silently if the provided coordinates are out of bounds.
pub fn cursor_move(x: u32, y: u32) {
    let pos = y * WIDTH + x;

    unsafe {
        outb(0x3D4, 0x0F);
        outb(0x3D5, pos as u8);
        outb(0x3D4, 0x0E);
        outb(0x3D5, (pos >> 8) as u8);
    }
}

/// A character in the VGA buffer.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct VgaChar(NonZeroU8);

impl VgaChar {
    /// The character ' '.
    pub const SPACE: Self = Self(unsafe { NonZeroU8::new_unchecked(b' ') });

    /// Attempts to convert a [`char`] to a [`VgaChar`].
    pub const fn from_char(c: char) -> Option<Self> {
        match c {
            _ if c.is_ascii_graphic() => Some(Self(unsafe { NonZeroU8::new_unchecked(c as u8) })),
            ' ' => Some(Self::SPACE),
            _ => None,
        }
    }

    /// Returns the underlying character as a byte.
    #[inline(always)]
    pub fn as_u8(&self) -> u8 {
        self.0.get()
    }
}

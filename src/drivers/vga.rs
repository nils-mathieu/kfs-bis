//! This module provides a simple VGA driver for writing characters to the screen.

use core::fmt::Debug;
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
#[repr(u8)]
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

impl Color {
    /// Returns an iterator over all available colors.
    #[inline]
    pub fn iter_all() -> impl Iterator<Item = Self> {
        (0u8..=15u8).map(|i| unsafe { core::mem::transmute(i) })
    }
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
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VgaChar(NonZeroU8);

impl VgaChar {
    /// Returns the underlying character as a byte.
    #[inline(always)]
    pub fn as_u8(self) -> u8 {
        self.0.get()
    }

    /// Returns an iterator over all available characters.
    #[inline]
    pub fn iter_all() -> impl Iterator<Item = Self> {
        (1..=255).map(|x| Self(unsafe { NonZeroU8::new_unchecked(x) }))
    }
}

impl Debug for VgaChar {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self.as_char(), f)
    }
}

/// Declares a set of VGA character constants for the [`VgaChar`] type.
macro_rules! declare_vga_chars {
    ( $( $character:literal => $( $name:ident )? ($value:literal); )* ) => {
        impl VgaChar {
            $($(
                #[doc = concat!("The `", stringify!($name), "` character.")]
                pub const $name: Self = Self(unsafe { NonZeroU8::new_unchecked($value) });
            )?)*

            /// Returns the [`VgaChar`] associated with the provided character.
            ///
            /// If the character does not map to any known VGA character, this function returns
            /// `None`.
            pub fn from_char(c: char) -> Option<Self> {
                match c {
                    $( $character => Some(Self(unsafe { NonZeroU8::new_unchecked($value) })), )*
                    _ => None,
                }
            }

            /// Returns the unicode character associated with the VGA character.
            pub fn as_char(self) -> char {
                match self.as_u8() {
                    $( $value => $character, )*
                    _ => '\0',
                }
            }
        }
    };
}

declare_vga_chars! {
    '☺' => FACE(0x01);
    '☻' => FILLED_FACE(0x02);
    '♥' => HEART(0x03);
    '♦' => DIAMOND(0x04);
    '♣' => CLUB(0x05);
    '♠' => SPADE(0x06);
    '\u{2022}' => BULLET(0x07);
    '◘' => INVERSE_BULLET(0x08);
    '○' => CIRCLE(0x09);
    '◙' => INVERSE_CIRCLE(0x0A);
    '♂' => MALE(0x0B);
    '♀' => FEMALE(0x0C);
    '♪' => NOTE(0x0D);
    '♫' => DOUBLE_NOTE(0x0E);
    '☼' => SUN(0x0F);
    '►' => RIGHT_TRIANGLE(0x10);
    '◄' => LEFT_TRIANGLE(0x11);
    '↕' => UP_DOWN_ARROW(0x12);
    '‼' => DOUBLE_EXCLAMATION(0x13);
    '¶' => PARAGRAPH(0x14);
    '§' => SECTION(0x15);
    '▬' => BLACK_RECTANGLE(0x16);
    '↨' => UP_DOWN_ARROW_WITH_BASE(0x17);
    '↑' => UP_ARROW(0x18);
    '↓' => DOWN_ARROW(0x19);
    '→' => RIGHT_ARROW(0x1A);
    '←' => LEFT_ARROW(0x1B);
    '∟' => RIGHT_ANGLE(0x1C);
    '↔' => LEFT_RIGHT_ARROW(0x1D);
    '▲' => UP_TRIANGLE(0x1E);
    '▼' => DOWN_TRIANGLE(0x1F);
    ' ' => SPACE(0x20);
    '!' => EXCLAMATION(0x21);
    '"' => DOUBLE_QUOTE(0x22);
    '#' => HASH(0x23);
    '$' => DOLLAR(0x24);
    '%' => PERCENT(0x25);
    '&' => AMPERSAND(0x26);
    '\'' => QUOTE(0x27);
    '(' => LEFT_PARENTHESIS(0x28);
    ')' => RIGHT_PARENTHESIS(0x29);
    '*' => ASTERISK(0x2A);
    '+' => PLUS(0x2B);
    ',' => COMMA(0x2C);
    '-' => MINUS(0x2D);
    '.' => PERIOD(0x2E);
    '/' => SLASH(0x2F);
    '0' => ZERO(0x30);
    '1' => ONE(0x31);
    '2' => TWO(0x32);
    '3' => THREE(0x33);
    '4' => FOUR(0x34);
    '5' => FIVE(0x35);
    '6' => SIX(0x36);
    '7' => SEVEN(0x37);
    '8' => EIGHT(0x38);
    '9' => NINE(0x39);
    ':' => COLON(0x3A);
    ';' => SEMICOLON(0x3B);
    '<' => LESS_THAN(0x3C);
    '=' => EQUALS(0x3D);
    '>' => GREATER_THAN(0x3E);
    '?' => QUESTION(0x3F);
    '@' => AT(0x40);
    'A' => A(0x41);
    'B' => B(0x42);
    'C' => C(0x43);
    'D' => D(0x44);
    'E' => E(0x45);
    'F' => F(0x46);
    'G' => G(0x47);
    'H' => H(0x48);
    'I' => I(0x49);
    'J' => J(0x4A);
    'K' => K(0x4B);
    'L' => L(0x4C);
    'M' => M(0x4D);
    'N' => N(0x4E);
    'O' => O(0x4F);
    'P' => P(0x50);
    'Q' => Q(0x51);
    'R' => R(0x52);
    'S' => S(0x53);
    'T' => T(0x54);
    'U' => U(0x55);
    'V' => V(0x56);
    'W' => W(0x57);
    'X' => X(0x58);
    'Y' => Y(0x59);
    'Z' => Z(0x5A);
    '[' => LEFT_BRACKET(0x5B);
    '\\' => BACKSLASH(0x5C);
    ']' => RIGHT_BRACKET(0x5D);
    '^' => CARET(0x5E);
    '_' => UNDERSCORE(0x5F);
    '`' => BACKTICK(0x60);
    'a' => LOWER_A(0x61);
    'b' => LOWER_B(0x62);
    'c' => LOWER_C(0x63);
    'd' => LOWER_D(0x64);
    'e' => LOWER_E(0x65);
    'f' => LOWER_F(0x66);
    'g' => LOWER_G(0x67);
    'h' => LOWER_H(0x68);
    'i' => LOWER_I(0x69);
    'j' => LOWER_J(0x6A);
    'k' => LOWER_K(0x6B);
    'l' => LOWER_L(0x6C);
    'm' => LOWER_M(0x6D);
    'n' => LOWER_N(0x6E);
    'o' => LOWER_O(0x6F);
    'p' => LOWER_P(0x70);
    'q' => LOWER_Q(0x71);
    'r' => LOWER_R(0x72);
    's' => LOWER_S(0x73);
    't' => LOWER_T(0x74);
    'u' => LOWER_U(0x75);
    'v' => LOWER_V(0x76);
    'w' => LOWER_W(0x77);
    'x' => LOWER_X(0x78);
    'y' => LOWER_Y(0x79);
    'z' => LOWER_Z(0x7A);
    '{' => LEFT_BRACE(0x7B);
    '|' => PIPE(0x7C);
    '}' => RIGHT_BRACE(0x7D);
    '~' => TILDE(0x7E);
    '⌂' => HOUSE(0x7F);
    'Ç' => CAPITAL_C_CEDILLA(0x80);
    'ü' => LOWER_U_UMLAUT(0x81);
    'é' => LOWER_E_ACUTE(0x82);
    'â' => LOWER_A_CIRCUMFLEX(0x83);
    'ä' => LOWER_A_UMLAUT(0x84);
    'à' => LOWER_A_GRAVE(0x85);
    'å' => LOWER_A_RING(0x86);
    'ç' => LOWER_C_CEDILLA(0x87);
    'ê' => LOWER_E_CIRCUMFLEX(0x88);
    'ë' => LOWER_E_UMLAUT(0x89);
    'è' => LOWER_E_GRAVE(0x8A);
    'ï' => LOWER_I_UMLAUT(0x8B);
    'î' => LOWER_I_CIRCUMFLEX(0x8C);
    'ì' => LOWER_I_GRAVE(0x8D);
    'Ä' => CAPITAL_A_UMLAUT(0x8E);
    'Å' => CAPITAL_A_RING(0x8F);
    'É' => CAPITAL_E_ACUTE(0x90);
    'æ' => LOWER_AE(0x91);
    'Æ' => CAPITAL_AE(0x92);
    'ô' => LOWER_O_CIRCUMFLEX(0x93);
    'ö' => LOWER_O_UMLAUT(0x94);
    'ò' => LOWER_O_GRAVE(0x95);
    'û' => LOWER_U_CIRCUMFLEX(0x96);
    'ù' => LOWER_U_GRAVE(0x97);
    'ÿ' => LOWER_Y_UMLAUT(0x98);
    'Ö' => CAPITAL_O_UMLAUT(0x99);
    'Ü' => CAPITAL_U_UMLAUT(0x9A);
    '¢' => CENT(0x9B);
    '£' => POUND(0x9C);
    '¥' => YEN(0x9D);
    '₧' => PESETA(0x9E);
    'ƒ' => F_HOOK(0x9F);
    'á' => LOWER_A_ACUTE(0xA0);
    'í' => LOWER_I_ACUTE(0xA1);
    'ó' => LOWER_O_ACUTE(0xA2);
    'ú' => LOWER_U_ACUTE(0xA3);
    'ñ' => LOWER_N_TILDE(0xA4);
    'Ñ' => CAPITAL_N_TILDE(0xA5);
    'ª' => FEMININE_ORDINAL(0xA6);
    'º' => MASCULINE_ORDINAL(0xA7);
    '¿' => INVERTED_QUESTION_MARK(0xA8);
    '⌐' => REVERSED_NOT_SIGN(0xA9);
    '¬' => NOT_SIGN(0xAA);
    '½' => ONE_HALF(0xAB);
    '¼' => ONE_QUARTER(0xAC);
    '¡' => INVERTED_EXCLAMATION_MARK(0xAD);
    '«' => LEFT_DOUBLE_ANGLE_QUOTE(0xAE);
    '»' => RIGHT_DOUBLE_ANGLE_QUOTE(0xAF);
    '░' => LIGHT_SHADE(0xB0);
    '▒' => MEDIUM_SHADE(0xB1);
    '▓' => DARK_SHADE(0xB2);
    '│' => (0xB3);
    '┤' => (0xB4);
    '╡' => (0xB5);
    '╢' => (0xB6);
    '╖' => (0xB7);
    '╕' => (0xB8);
    '╣' => (0xB9);
    '║' => (0xBA);
    '╗' => (0xBB);
    '╝' => (0xBC);
    '╜' => (0xBD);
    '╛' => (0xBE);
    '┐' => (0xBF);
    '└' => (0xC0);
    '┴' => (0xC1);
    '┬' => (0xC2);
    '├' => (0xC3);
    '─' => (0xC4);
    '┼' => (0xC5);
    '╞' => (0xC6);
    '╟' => (0xC7);
    '╚' => (0xC8);
    '╔' => (0xC9);
    '╩' => (0xCA);
    '╦' => (0xCB);
    '╠' => (0xCC);
    '═' => (0xCD);
    '╬' => (0xCE);
    '╧' => (0xCF);
    '╨' => (0xD0);
    '╤' => (0xD1);
    '╥' => (0xD2);
    '╙' => (0xD3);
    '╘' => (0xD4);
    '╒' => (0xD5);
    '╓' => (0xD6);
    '╫' => (0xD7);
    '╪' => (0xD8);
    '┘' => (0xD9);
    '┌' => (0xDA);
    '█' => FULL_BLOCK(0xDB);
    '▄' => LOWER_HALF_BLOCK(0xDC);
    '▌' => LEFT_HALF_BLOCK(0xDD);
    '▐' => RIGHT_HALF_BLOCK(0xDE);
    '▀' => UPPER_HALF_BLOCK(0xDF);
    'α' => LOWER_ALPHA(0xE0);
    'ß' => LOWER_BETA(0xE1);
    'Γ' => CAPITAL_GAMMA(0xE2);
    'π' => LOWER_PI(0xE3);
    'Σ' => CAPITAL_SIGMA(0xE4);
    'σ' => LOWER_SIGMA(0xE5);
    'µ' => MICRO(0xE6);
    'τ' => LOWER_TAU(0xE7);
    'Φ' => CAPITAL_PHI(0xE8);
    'Θ' => CAPITAL_THETA(0xE9);
    'Ω' => CAPITAL_OMEGA(0xEA);
    'δ' => LOWER_DELTA(0xEB);
    '∞' => INFINITY(0xEC);
    'φ' => LOWER_PHI(0xED);
    'ε' => LOWER_EPSILON(0xEE);
    '∩' => INTERSECTION(0xEF);
    '≡' => IDENTICAL_TO(0xF0);
    '±' => PLUS_MINUS(0xF1);
    '≥' => GREATER_THAN_OR_EQUAL_TO(0xF2);
    '≤' => LESS_THAN_OR_EQUAL_TO(0xF3);
    '⌠' => TOP_HALF_INTEGRAL(0xF4);
    '⌡' => BOTTOM_HALF_INTEGRAL(0xF5);
    '÷' => DIVIDE(0xF6);
    '≈' => ALMOST_EQUAL_TO(0xF7);
    '°' => DEGREE(0xF8);
    '\u{2219}' => (0xF9);
    '\u{00B7}' => MIDDLE_DOT(0xFA);
    '√' => SQUARE_ROOT(0xFB);
    'ⁿ' => SUPERSCRIPT_N(0xFC);
    '²' => SUPERSCRIPT_2(0xFD);
    '■' => SOLID_BLOCK(0xFE);
}

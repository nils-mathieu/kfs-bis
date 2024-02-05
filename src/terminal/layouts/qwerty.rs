use super::Modifiers;

/// The current state of the state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    /// The state machine is in the neutral state. No sequence of scancode has been
    /// generated yet.
    Neutral,
    /// The E0 escape code has been received.
    E0,
}

/// Contains the state required to convert scan-codes into text.
pub struct Qwerty {
    /// The state of key modifiers.
    modifiers: Modifiers,
    /// The current state of the state machine.
    state: State,

    /// Whether the numlock key is currently pressed. This is necessary to avoid toggling
    /// the NUM_LOCK state on key repeats.
    numlock_repeating: bool,
    /// Like `numlock`, but for the capslock key.
    capslock_repeating: bool,
}

impl Qwerty {
    /// Returns a new instance of the [`Qwerty`] struct.
    pub const fn new() -> Self {
        Self {
            modifiers: Modifiers::empty(),
            state: State::Neutral,
            numlock_repeating: false,
            capslock_repeating: false,
        }
    }

    /// Returns the current state of the modifiers.
    #[inline(always)]
    pub fn modifiers(&self) -> Modifiers {
        self.modifiers
    }

    /// Advances the state of the state machine with a new scan-code. If a character can
    /// be produced, it is returned in a [`Some(_)`] variant.
    ///
    /// If no character could be produced, [`None`] is returned instead.
    pub fn advance(&mut self, scancode: u8) -> Option<char> {
        use State::*;

        let st = self.state;

        // Parse the current escape sequence.
        self.state = match (st, scancode) {
            (Neutral, 0xE0) => E0,
            _ => Neutral,
        };

        match (st, scancode) {
            // Update modifiers.
            (Neutral, 0x2A) => {
                self.modifiers.insert(Modifiers::LEFT_SHIFT);
                None
            }
            (Neutral, 0xAA) => {
                self.modifiers.remove(Modifiers::LEFT_SHIFT);
                None
            }
            (Neutral, 0x36) => {
                self.modifiers.insert(Modifiers::RIGHT_SHIFT);
                None
            }
            (Neutral, 0xB6) => {
                self.modifiers.remove(Modifiers::RIGHT_SHIFT);
                None
            }
            (Neutral, 0x1D) => {
                self.modifiers.insert(Modifiers::LEFT_CONTROL);
                None
            }
            (Neutral, 0x9D) => {
                self.modifiers.remove(Modifiers::LEFT_CONTROL);
                None
            }
            (Neutral, 0x3A) => {
                if !self.capslock_repeating {
                    self.capslock_repeating = true;
                    self.modifiers.toggle(Modifiers::CAPS_LOCK);
                }
                None
            }
            (Neutral, 0xBA) => {
                self.capslock_repeating = false;
                None
            }
            (E0, 0x1D) => {
                self.modifiers.insert(Modifiers::RIGHT_CONTROL);
                None
            }
            (E0, 0x9D) => {
                self.modifiers.remove(Modifiers::RIGHT_CONTROL);
                None
            }
            (Neutral, 0x38) => {
                self.modifiers.insert(Modifiers::LEFT_ALT);
                None
            }
            (Neutral, 0xB8) => {
                self.modifiers.remove(Modifiers::LEFT_ALT);
                None
            }
            (E0, 0x38) => {
                self.modifiers.insert(Modifiers::RIGHT_ALT);
                None
            }
            (E0, 0xB8) => {
                self.modifiers.remove(Modifiers::RIGHT_ALT);
                None
            }
            (Neutral, 0x45) => {
                if !self.numlock_repeating {
                    self.numlock_repeating = true;
                    self.modifiers.toggle(Modifiers::NUM_LOCK);
                }
                None
            }
            (Neutral, 0xC5) => {
                self.numlock_repeating = false;
                None
            }
            // Printable characters.
            (Neutral, 0x02) if !self.modifiers.shifted() => Some('1'),
            (Neutral, 0x02) if self.modifiers.shifted() => Some('!'),
            (Neutral, 0x03) if !self.modifiers.shifted() => Some('2'),
            (Neutral, 0x03) if self.modifiers.shifted() => Some('@'),
            (Neutral, 0x04) if !self.modifiers.shifted() => Some('3'),
            (Neutral, 0x04) if self.modifiers.shifted() => Some('#'),
            (Neutral, 0x05) if !self.modifiers.shifted() => Some('4'),
            (Neutral, 0x05) if self.modifiers.shifted() => Some('$'),
            (Neutral, 0x06) if !self.modifiers.shifted() => Some('5'),
            (Neutral, 0x06) if self.modifiers.shifted() => Some('%'),
            (Neutral, 0x07) if !self.modifiers.shifted() => Some('6'),
            (Neutral, 0x07) if self.modifiers.shifted() => Some('^'),
            (Neutral, 0x08) if !self.modifiers.shifted() => Some('7'),
            (Neutral, 0x08) if self.modifiers.shifted() => Some('&'),
            (Neutral, 0x09) if !self.modifiers.shifted() => Some('8'),
            (Neutral, 0x09) if self.modifiers.shifted() => Some('*'),
            (Neutral, 0x0A) if !self.modifiers.shifted() => Some('9'),
            (Neutral, 0x0A) if self.modifiers.shifted() => Some('('),
            (Neutral, 0x0B) if !self.modifiers.shifted() => Some('0'),
            (Neutral, 0x0B) if self.modifiers.shifted() => Some(')'),
            (Neutral, 0x0C) if !self.modifiers.shifted() => Some('-'),
            (Neutral, 0x0C) if self.modifiers.shifted() => Some('_'),
            (Neutral, 0x0D) if !self.modifiers.shifted() => Some('='),
            (Neutral, 0x0D) if self.modifiers.shifted() => Some('+'),
            (Neutral, 0x10) if !self.modifiers.shifted() => Some('q'),
            (Neutral, 0x10) if self.modifiers.shifted() => Some('Q'),
            (Neutral, 0x11) if !self.modifiers.shifted() => Some('w'),
            (Neutral, 0x11) if self.modifiers.shifted() => Some('W'),
            (Neutral, 0x12) if !self.modifiers.shifted() => Some('e'),
            (Neutral, 0x12) if self.modifiers.shifted() => Some('E'),
            (Neutral, 0x13) if !self.modifiers.shifted() => Some('r'),
            (Neutral, 0x13) if self.modifiers.shifted() => Some('R'),
            (Neutral, 0x14) if !self.modifiers.shifted() => Some('t'),
            (Neutral, 0x14) if self.modifiers.shifted() => Some('T'),
            (Neutral, 0x15) if !self.modifiers.shifted() => Some('y'),
            (Neutral, 0x15) if self.modifiers.shifted() => Some('Y'),
            (Neutral, 0x16) if !self.modifiers.shifted() => Some('u'),
            (Neutral, 0x16) if self.modifiers.shifted() => Some('U'),
            (Neutral, 0x17) if !self.modifiers.shifted() => Some('i'),
            (Neutral, 0x17) if self.modifiers.shifted() => Some('I'),
            (Neutral, 0x18) if !self.modifiers.shifted() => Some('o'),
            (Neutral, 0x18) if self.modifiers.shifted() => Some('O'),
            (Neutral, 0x19) if !self.modifiers.shifted() => Some('p'),
            (Neutral, 0x19) if self.modifiers.shifted() => Some('P'),
            (Neutral, 0x1A) if !self.modifiers.shifted() => Some('['),
            (Neutral, 0x1A) if self.modifiers.shifted() => Some('{'),
            (Neutral, 0x1B) if !self.modifiers.shifted() => Some(']'),
            (Neutral, 0x1B) if self.modifiers.shifted() => Some('}'),
            (Neutral, 0x2B) if !self.modifiers.shifted() => Some('\\'),
            (Neutral, 0x2B) if self.modifiers.shifted() => Some('|'),
            (Neutral, 0x1E) if !self.modifiers.shifted() => Some('a'),
            (Neutral, 0x1E) if self.modifiers.shifted() => Some('A'),
            (Neutral, 0x1F) if !self.modifiers.shifted() => Some('s'),
            (Neutral, 0x1F) if self.modifiers.shifted() => Some('S'),
            (Neutral, 0x20) if !self.modifiers.shifted() => Some('d'),
            (Neutral, 0x20) if self.modifiers.shifted() => Some('D'),
            (Neutral, 0x21) if !self.modifiers.shifted() => Some('f'),
            (Neutral, 0x21) if self.modifiers.shifted() => Some('F'),
            (Neutral, 0x22) if !self.modifiers.shifted() => Some('g'),
            (Neutral, 0x22) if self.modifiers.shifted() => Some('G'),
            (Neutral, 0x23) if !self.modifiers.shifted() => Some('h'),
            (Neutral, 0x23) if self.modifiers.shifted() => Some('H'),
            (Neutral, 0x24) if !self.modifiers.shifted() => Some('j'),
            (Neutral, 0x24) if self.modifiers.shifted() => Some('J'),
            (Neutral, 0x25) if !self.modifiers.shifted() => Some('k'),
            (Neutral, 0x25) if self.modifiers.shifted() => Some('K'),
            (Neutral, 0x26) if !self.modifiers.shifted() => Some('l'),
            (Neutral, 0x26) if self.modifiers.shifted() => Some('L'),
            (Neutral, 0x27) if !self.modifiers.shifted() => Some(';'),
            (Neutral, 0x27) if self.modifiers.shifted() => Some(':'),
            (Neutral, 0x28) if !self.modifiers.shifted() => Some('\''),
            (Neutral, 0x28) if self.modifiers.shifted() => Some('"'),
            (Neutral, 0x29) if !self.modifiers.shifted() => Some('`'),
            (Neutral, 0x29) if self.modifiers.shifted() => Some('~'),
            (Neutral, 0x2C) if !self.modifiers.shifted() => Some('z'),
            (Neutral, 0x2C) if self.modifiers.shifted() => Some('Z'),
            (Neutral, 0x2D) if !self.modifiers.shifted() => Some('x'),
            (Neutral, 0x2D) if self.modifiers.shifted() => Some('X'),
            (Neutral, 0x2E) if !self.modifiers.shifted() => Some('c'),
            (Neutral, 0x2E) if self.modifiers.shifted() => Some('C'),
            (Neutral, 0x2F) if !self.modifiers.shifted() => Some('v'),
            (Neutral, 0x2F) if self.modifiers.shifted() => Some('V'),
            (Neutral, 0x30) if !self.modifiers.shifted() => Some('b'),
            (Neutral, 0x30) if self.modifiers.shifted() => Some('B'),
            (Neutral, 0x31) if !self.modifiers.shifted() => Some('n'),
            (Neutral, 0x31) if self.modifiers.shifted() => Some('N'),
            (Neutral, 0x32) if !self.modifiers.shifted() => Some('m'),
            (Neutral, 0x32) if self.modifiers.shifted() => Some('M'),
            (Neutral, 0x33) if !self.modifiers.shifted() => Some(','),
            (Neutral, 0x33) if self.modifiers.shifted() => Some('<'),
            (Neutral, 0x34) if !self.modifiers.shifted() => Some('.'),
            (Neutral, 0x34) if self.modifiers.shifted() => Some('>'),
            (Neutral, 0x35) if !self.modifiers.shifted() => Some('/'),
            (E0, 0x35) => Some('/'),
            (Neutral, 0x35) if self.modifiers.shifted() => Some('?'),
            (Neutral, 0x47) if self.modifiers.num_locked() => Some('7'),
            (Neutral, 0x48) if self.modifiers.num_locked() => Some('8'),
            (Neutral, 0x49) if self.modifiers.num_locked() => Some('9'),
            (Neutral, 0x4B) if self.modifiers.num_locked() => Some('4'),
            (Neutral, 0x4C) if self.modifiers.num_locked() => Some('5'),
            (Neutral, 0x4D) if self.modifiers.num_locked() => Some('6'),
            (Neutral, 0x4F) if self.modifiers.num_locked() => Some('1'),
            (Neutral, 0x50) if self.modifiers.num_locked() => Some('2'),
            (Neutral, 0x51) if self.modifiers.num_locked() => Some('3'),
            (Neutral, 0x52) if self.modifiers.num_locked() => Some('0'),
            (Neutral, 0x53) if self.modifiers.num_locked() => Some('.'),
            // Non-printable keys
            (Neutral, 0x39) => Some(' '),
            (Neutral | E0, 0x1C) => Some('\n'),
            (Neutral, 0x0E) => Some('\x08'),
            (Neutral, 0x0F) => Some('\t'),
            _ => None,
        }
    }
}

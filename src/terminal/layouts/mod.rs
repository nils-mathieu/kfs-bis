//! This module contains the keyboard layouts supported by the kernel.

mod qwerty;

pub use self::qwerty::Qwerty;

use bitflags::bitflags;

bitflags! {
    /// Keeps track of the state of certain special keys, such as CONTROL or SHIFT.
    #[derive(Default, Clone, Copy)]
    pub struct Modifiers: u8 {
        /// Whether the left **CONTROL** key is currently pressed.
        const LEFT_CONTROL = 1 << 0;
        /// Whether the right **CONTROL** key is currently pressed.
        const RIGHT_CONTROL = 1 << 1;
        /// Whether the left **SHIFT** key is currently pressed.
        const LEFT_SHIFT = 1 << 2;
        /// Whether the right **SHIFT** key is currently pressed.
        const RIGHT_SHIFT = 1 << 3;
        /// Whether the **CAPS LOCK** key is currently active.
        const CAPS_LOCK = 1 << 4;
        /// Whether the left **ALT** key is currently pressed.
        const LEFT_ALT = 1 << 5;
        /// Whether the right **ALT** key is currently pressed.
        const RIGHT_ALT = 1 << 6;
        /// Whether the **NUM LOCK** key is currently active.
        ///
        /// When it is active, the numeric keypad is in number mode. When it is inactive, the
        /// numeric keypad is in arrow mode.
        const NUM_LOCK = 1 << 7;
    }
}

impl Modifiers {
    /// Returns whether the keys are currently shifted.
    #[inline]
    pub fn shifted(&self) -> bool {
        self.has_shift() ^ self.intersects(Modifiers::CAPS_LOCK)
    }

    /// Whether any of the control keys are currently pressed.
    #[inline]
    pub fn has_control(&self) -> bool {
        self.intersects(Modifiers::LEFT_CONTROL | Modifiers::RIGHT_CONTROL)
    }

    /// Whether any of the shift keys are currently pressed.
    ///
    /// # Remarks
    ///
    /// This does not take `CAPS_LOCK` into account. See [`shifted`](Self::shifted) for that.
    #[inline]
    pub fn has_shift(&self) -> bool {
        self.intersects(Modifiers::LEFT_SHIFT | Modifiers::RIGHT_SHIFT)
    }

    /// Whether any of the alt keys are currently pressed.
    #[inline]
    pub fn has_alt(&self) -> bool {
        self.intersects(Modifiers::LEFT_ALT | Modifiers::RIGHT_ALT)
    }

    /// Returns whether the NUM LOCK key is currently active.
    #[inline]
    pub fn num_locked(&self) -> bool {
        self.intersects(Modifiers::NUM_LOCK)
    }
}

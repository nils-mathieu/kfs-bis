use super::instr::{cli, read_rflags, sti, RFlags};

/// A simple type that automatically restores interrupt with dropped.
pub struct RestoreInterrupts;

impl RestoreInterrupts {
    /// Conditionally creates an instance of [`RestoreInterrupts`] if interrupts are
    /// currently enabled.
    ///
    /// If interrupts are enabled, they are automatically disabled by this function and an instance
    /// of [`RestoreInterrupts`] is returned. If interrupts are already disabled, this function
    /// returns `None`.
    pub fn without_interrupts() -> Option<Self> {
        if read_rflags().intersects(RFlags::INTERRUPT) {
            cli();
            Some(Self)
        } else {
            None
        }
    }
}

impl Drop for RestoreInterrupts {
    fn drop(&mut self) {
        sti();
    }
}

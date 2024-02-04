//! An implementation of a PS/2 controller.

use bitflags::bitflags;

use crate::utility::instr::{inb, outb};

/// Reads the status register of the PS/2 controller.
#[inline]
pub fn status() -> PS2Status {
    let raw = unsafe { inb(0x64) };
    PS2Status::from_bits_retain(raw)
}

/// Returns whether the output buffer of the PS/2 controller is full.
///
/// When this function returns `true`, it is okay to read from the data
/// register.
#[inline]
pub fn is_output_buffer_full() -> bool {
    status().intersects(PS2Status::OUTPUT_BUFFER_FULL)
}

/// Sends a command to the PS/2 controller.
#[inline]
pub fn command(cmd: u8) {
    unsafe { outb(0x64, cmd) }
}

/// Reads the data register of the PS/2 controller.
///
/// # Remarks
///
/// This function does not check whether the output buffer actually contain any meaningful data.
/// The caller is responsible for checking the status register before calling this function, or
/// having another means of knowing whether the output buffer contains meaningful data, such as
/// having received an interrupt from the PS/2 controller.
#[inline]
pub fn read_data() -> u8 {
    unsafe { inb(0x60) }
}

/// Sends data to the PS/2 controller.
#[inline]
pub fn write_data(data: u8) {
    unsafe { outb(0x60, data) }
}

bitflags! {
    /// Represents the status register of the PS/2 controller.
    #[derive(Clone, Copy, Debug)]
    pub struct PS2Status: u8 {
        /// Indicates that the output buffer of the controller is full.
        ///
        /// This bit must be set when attempting to read the data register.
        const OUTPUT_BUFFER_FULL = 1 << 0;

        /// Indicates that the input buffer of the controller is full.
        ///
        /// This bit must be clear when attempting to write to the data or command register.
        const INPUT_BUFFER_FULL = 1 << 1;

        /// This bit is meant to be cleared when the controller is reset. It is then set again
        /// by the firmware.
        const SYSTEM = 1 << 2;

        /// When this bit is set, the data written to the input buffer is meant for the PS/2
        /// controller command.
        ///
        /// When this bit is clear, the data written to the input buffer is meant for the PS/2
        /// device.
        const COMMAND = 1 << 3;

        /// Indicates that the data present in the output buffer is from the second PS/2 port.
        ///
        /// # Remarks
        ///
        /// This is apparently not reliable on some hardware, as the meaning of this bit
        /// is not well-defined.
        ///
        /// Works well enough :p
        const AUX_OUTPUT_BUFFER_FULL = 1 << 5;

        /// Indicates that a timeout error has occurred.
        const TIMEOUT_ERROR = 1 << 6;
        /// Indicates that a parity error has occurred.
        const PARITY_ERROR = 1 << 7;
    }
}

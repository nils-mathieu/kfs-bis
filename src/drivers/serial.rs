//! A simple serial I/O driver.

use bitflags::bitflags;

use crate::utility::instr::{inb, outb, pause};

/// Base address of the COM1 serial port used in this module for logging.
const PORT: u16 = 0x3F8;

/// The register responsible for requesting the serial port to operate in interrupt (or polling)
/// mode.
///
/// See the [OSDev Wiki](https://wiki.osdev.org/Serial_Ports#Interrupt_enable_register).
const INTERRUPT_ENABLE: u16 = PORT + 1;

/// The line-control register.
///
/// This is used to configure the protocol of the serial port.
const LINE_CONTROL: u16 = PORT + 3;

/// The model-control register.
///
/// This is used to configure how the serial port is used.
const MODEM_CONTROL: u16 = PORT + 4;

/// The line-status register.
///
/// This is used to determine whether the serial port is ready to send more data, among
/// other things.
const LINE_STATUS: u16 = PORT + 5;

/// The bit responsible for enabling the DLAB (Divisor Latch Access Bit) in the line-control
/// register.
const DLAB: u8 = 0x80;

/// The parity bits in the line-control register that indicate that no parity bit should be used
/// in the protocol.
const PARITY_NONE: u8 = 0x00;

/// The bits in the line-control register that indicate that the serial port should use 8-bit
/// of data.
const DATA_LENGTH_8BITS: u8 = 0x03;

/// The bits in the line-control register that indicate that the serial port should use 1 stop
/// bit.
const STOP_BIT_1: u8 = 0x00;

/// A good default value for the line-control register. Basically every single emulator ever
/// uses those settings, which increases the chances of being able to use the serial port
/// without too much hassle.
const DEFAULT_LINE_CONTROL: u8 = PARITY_NONE | DATA_LENGTH_8BITS | STOP_BIT_1;

/// Controls the DTR pin when set on the modem-control register.
const DATA_TERMINAL_READY: u8 = 0x01;

/// Controls the RTS pin when set on the modem-control register.
const REQUEST_TO_SEND: u8 = 0x02;

/// Initializes the serial port driver.
pub fn init() {
    // The following is adapted from the OSDev Wiki (this has to be the most copy-pasted code
    // of the whole wiki lol).
    //
    //     https://wiki.osdev.org/Serial_Ports#Initialization
    //     https://en.wikipedia.org/wiki/Serial_port
    //

    // Make sure that the serial port won't attempt to send interrupts to the CPU. If we need
    // to determine whether the serial port is ready to send data, we will poll it instead.
    disable_interrupts();

    // Set the baud rate divisor to 3 (for a total of 38400 bauds).
    // This is generally a good default for the use-case of simply logging messages.
    set_baud_rate_divisor(3);

    // Configure the serial port to use the default settings.
    set_default_line_control();

    // Enable the FIFO buffer of the serial port, with a 14-byte threshold.
    enable_fifo();

    // Finish the handshake with the serial port by writing the `DATA_TERMINAL_READY` and
    // `REQUEST_TO_SEND` bits to the modem-control register.
    // This is needed to actually enable the serial port.
    finish_handshake();
}

bitflags! {
    /// Defines the status bits for the serial port.
    #[derive(Clone, Copy, Debug)]
    pub struct SerialStatus: u8 {
        /// Indicates that the transmitter is not doing anything. When this bit is set,
        /// it's possible to write to the serial port without risking to lose data.
        const TRANSMITTER_EMPTY = 0x20;
    }
}

/// Returns the current status of the serial port.
#[inline]
pub fn status() -> SerialStatus {
    let raw = unsafe { inb(LINE_STATUS) };
    SerialStatus::from_bits_retain(raw)
}

/// Returns whether the serial port is ready to send more data.
#[inline]
pub fn ready_to_send() -> bool {
    status().intersects(SerialStatus::TRANSMITTER_EMPTY)
}

/// Writes a byte to the serial port, eventually waiting for the transmitter to be ready
/// to send more data.
pub fn write_byte(byte: u8) {
    while !ready_to_send() {
        pause();
    }

    unsafe {
        outb(PORT, byte);
    }
}

/// Writes the provided bytes through the serial port.
pub fn write_bytes(bytes: &[u8]) {
    bytes.iter().copied().for_each(write_byte);
}

/// A simple struct that implements [`core::fmt::Write`].
#[derive(Debug, Clone, Copy)]
pub struct Serial;

impl core::fmt::Write for Serial {
    #[inline]
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        write_bytes(s.as_bytes());
        Ok(())
    }
}

/// Ensures that the serial port won't attempt to send interrupts to the CPU.
fn disable_interrupts() {
    unsafe {
        outb(INTERRUPT_ENABLE, 0x00);
    }
}

/// Sets the baud-rate divisor of the serial port.
///
/// # Remarks
///
/// This function clobbers the line-control register.
fn set_baud_rate_divisor(divisor: u16) {
    unsafe {
        outb(LINE_CONTROL, DLAB);

        // +0 is the low byte
        // +1 is the high byte
        outb(PORT, divisor as u8);
        outb(PORT + 1, (divisor >> 8) as u8);
    }
}

/// Configures the protocol of the serial port to use the default settings.
fn set_default_line_control() {
    unsafe {
        outb(LINE_CONTROL, DEFAULT_LINE_CONTROL);
    }
}

/// Enables the FIFO buffer of the serial port, with a 14-byte threshold.
fn enable_fifo() {
    // MISSING_DOC: Not sure where to find the documentation for this.
    // This line is straight up copied from the OSDev Wiki, but I'm not sure
    // where they got it from.

    unsafe {
        outb(MODEM_CONTROL, 0xC7);
    }
}

/// Finish the handshake with the serial port by writing the `DATA_TERMINAL_READY` and
/// `REQUEST_TO_SEND` bits to the modem-control register.
fn finish_handshake() {
    unsafe {
        outb(MODEM_CONTROL, DATA_TERMINAL_READY | REQUEST_TO_SEND);
    }
}

/// Only used in the log macro.
#[doc(hidden)]
#[cfg(feature = "log_serial")]
#[inline]
pub fn __log(msg: core::fmt::Arguments) {
    let _ = core::fmt::Write::write_fmt(&mut Serial, msg);
}

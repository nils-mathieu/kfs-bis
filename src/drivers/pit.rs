//! A Programmable Interval Timer (PIT) driver.

use core::sync::atomic::AtomicU32;
use core::sync::atomic::Ordering::Relaxed;

use bitflags::bitflags;

use crate::log;
use crate::utility::instr::outb;

bitflags! {
    /// The command codes that can be sent to the PIT.
    struct PitCmd: u8 {
        /// Indicates that the PIT is configured to send a one-time interrupt on IRQ0.
        const CHANNEL_0 = 0b00 << 6;

        /// Data transfered from/to the PIT is read as a sequence of two bytes to make a 16-bit
        /// word.
        ///
        /// The low byte is sent first, followed by the high byte.
        const ACCESS_MODE_LO_HI = 0b11 << 4;

        /// Indicates that the PIT should send an interrupt at a certain frequency.
        const RATE_GENERATOR = 0b010 << 1;

    }
}

/// The command port of the PIT.
const COMMAND_PORT: u16 = 0x43;

/// The data port of the PIT.
const DATA_PORT: u16 = 0x40;

/// Sends a commant to the PIT.
#[inline]
fn command(cmd: PitCmd) {
    unsafe { outb(COMMAND_PORT, cmd.bits()) }
}

/// Writes to the data register of the PIT.
///
/// # Remarks
///
/// This function assumes that the PIT is configured with
/// access mode `ACCESS_MODE_LO_HI`.
#[inline]
fn set_reload_value(data: u16) {
    unsafe {
        outb(DATA_PORT, (data & 0xFF) as u8);
        outb(DATA_PORT, ((data >> 8) & 0xFF) as u8);
    }
}

//
// The following code is most translated from the OSDev Wiki:
//
// https://wiki.osdev.org/PIT
//

/// Computes the quotient of `num` divided by `denom`, rounding the result to the nearest integer.
#[inline]
fn divide_rounded(num: u64, denom: u64) -> u64 {
    (num + denom / 2) / denom
}

/// Computes the reload-value that should be used for the PIT in order to generate
/// the requested frequency.
///
/// `freq` is the number of interrupts that the PIT will generate, per second.
///
/// # Remarks
///
/// The actual frequency that the PIT will generate will be slightly different from the requested
/// frequency because of rounding errors.
///
/// One should re-compute the actual frequency that the PIT will generate using the reload
/// value returned by the function.
fn freq_to_reload_value(freq: u64) -> u64 {
    // Compute the reload value.
    if freq <= 18 {
        0x10000
    } else if freq >= 1193181 {
        1
    } else {
        // We're using 3579545 as the base frequency to mitigate rounding errors
        // (3579545 / 3 = 1193181.667) for extra accuracy.
        // This is a trick used by the wiki, it's clever.
        divide_rounded(3579545, 3 * freq)
    }
}

/// Computes the frequency that the PIT will generate with the specified reload value.
fn reload_value_to_freq(rl: u64) -> f64 {
    3579545.0 / (3.0 * rl as f64)
}

/// Computes the number of nanoseconds between two interrupts, for the provided
/// reload value.
fn reload_value_to_ns(rl: u64) -> u32 {
    // freq = 3579545 / (3 * rl)
    // ns   = 1e9 / freq
    //      = 3 * rl * 1e9 / 3579545
    let ret = divide_rounded(3 * rl * 1_000_000_000, 3579545);

    #[cfg(debug_assertions)]
    {
        ret.try_into()
            .expect("computed tick duration overflows a 32-bit integer")
    }
    #[cfg(not(debug_assertions))]
    {
        ret as u32
    }
}

/// See [`interval_ns`].
static INTERVAL_NS: AtomicU32 = AtomicU32::new(0);

/// Once the PIT has been initialized, this returns the number of nanoseconds between two
/// interrupts sent by the PIT.
///
/// If it was never initialized, this function returns 0.
#[inline]
pub fn interval_ns() -> u32 {
    INTERVAL_NS.load(Relaxed)
}

/// Initializes the PIT.
///
/// # Remarks
///
/// This function assumes that interrupts are currently disabled, ensuring that
/// the PIT won't generate an IRQ while it's not yet configured.
pub fn init() {
    log!("Initializing the Programmable Interval Timer (PIT)...\n");

    let reload_value = freq_to_reload_value(1000); // 1 ms

    log!(
        "PIT frequency: {:.2} Hz (rl = {})\n",
        reload_value_to_freq(reload_value),
        reload_value,
    );

    assert!(
        reload_value <= 0x1000,
        "Computed PIT reload value is too high ({})\n",
        reload_value,
    );

    INTERVAL_NS.store(reload_value_to_ns(reload_value), Relaxed);

    // Send the command to the PIT to configure it to send a one-time interrupt on IRQ0 when the
    // terminal count is reached.
    command(PitCmd::CHANNEL_0 | PitCmd::ACCESS_MODE_LO_HI | PitCmd::RATE_GENERATOR);
    set_reload_value(reload_value as u16);
}

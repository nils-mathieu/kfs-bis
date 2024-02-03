//! The driving code for the Programmable Interrupt Controller.

use bitflags::bitflags;

use crate::cpu::idt::PIC_OFFSET;
use crate::utility::instr::outb;

/// A PIC (Programmable Interrupt Controller).
struct Pic {
    /// The command port of the PIC.
    cmd: u16,
    /// The data port of the PIC.
    data: u16,
}

impl Pic {
    /// The first PIC.
    pub const MASTER: Self = Self {
        cmd: 0x20,
        data: 0x21,
    };

    /// The second PIC.
    pub const SLAVE: Self = Self {
        cmd: 0xA0,
        data: 0xA1,
    };

    /// Sends a command byte to the PIC.
    #[inline]
    pub fn command(self, cmd: u8) {
        unsafe { outb(self.cmd, cmd) }
    }

    /// Writes data to the PIC.
    #[inline]
    pub fn write(self, data: u8) {
        unsafe { outb(self.data, data) }
    }
}

/// Initializes the PIC.
pub fn init() {
    // ICW stands for "Initialization Command Word" btw.

    // Start the initialization sequence by sending the initialization command to both PICs.
    //
    // bit 0 - indicates that ICW4 is needed.
    // bit 1 - cascade mode (we're using a master/slave configuration).
    // bit 2 - call address interval (interval of 8)
    // bit 3 - edge triggered mode
    // bit 4 - start initialization sequence (this bit is required to start the initialization
    //         sequence).
    Pic::MASTER.command(0x11);
    wait_a_bit();
    Pic::SLAVE.command(0x11);
    wait_a_bit();

    // Indicate which vector offset the PICs should use.
    //
    // This is ICW2.
    Pic::MASTER.write(PIC_OFFSET);
    wait_a_bit();
    Pic::SLAVE.write(PIC_OFFSET + 8);
    wait_a_bit();

    // Tell the master PIC that there is a slave PIC at IRQ2 (0000 0100).
    //
    // This is ICW3 (master and slave don't have the same meaning at that point).
    Pic::MASTER.write(1 << 2);
    wait_a_bit();
    Pic::SLAVE.write(1 << 1);
    wait_a_bit();

    // Use 8086 mode instead of 8085 mode.
    //
    // This is ICW4 (we requested it in the first command).
    Pic::MASTER.write(0x01);
    wait_a_bit();
    Pic::SLAVE.write(0x01);
    wait_a_bit();
}

/// Send an END-OF-INTERRUPT command to the PIC for the provided IRQ.
#[inline]
pub fn end_of_interrupt(irq: Irq) {
    // EOI is bit 5 of the operation command word (OCW2).
    // That word is sent to the command register.

    if irq as u8 >= 8 {
        Pic::SLAVE.command(1 << 5);
    }

    Pic::MASTER.command(1 << 5);
}

/// Sets the IRQ mask for the PIC.
///
/// # Remarks
///
/// When a bit set in [`Irqs`], the corresponding IRQ is **disabled**. If a bit is not set,
/// then it will be enabled and will be able to trigger an interrupt.
#[inline]
pub fn set_irq_mask(masked_irqs: Irqs) {
    // OCW1 is the operation command word 1. It contains a mask
    // of the IRQs that should be disabled.
    // That word is sent to the data register.

    Pic::MASTER.write(masked_irqs.bits() as u8);
    Pic::SLAVE.write((masked_irqs.bits() >> 8) as u8);
}

/// Perform an operation that takes a bit of time to complete but has no side effects. This is
/// needed because some older machines are too fast for the PIC to keep up with, so we need to
/// wait a bit after sending a command to the PIC.
///
/// This function takes between 1 to 4 microseconds to complete.
#[inline]
fn wait_a_bit() {
    // Any unused port works for this. Linux uses 0x80, so we'll use that too. It's almost always
    // unused after boot.
    unsafe { outb(0x80, 0u8) };
}

/// A possible IRQ number.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Irq {
    /// IRQ 0.
    Timer,
    /// IRQ 1.
    Keyboard,
    /// IRQ 2.
    Cascade,
    /// IRQ 3.
    Com2,
    /// IRQ 4.
    Com1,
    /// IRQ 5.
    Lpt2,
    /// IRQ 6.
    Floppy,
    /// IRQ 7.
    Lpt1,
    /// IRQ 8.
    RealTimeClock,
    /// IRQ 9.
    Periph1,
    /// IRQ 10.
    Periph2,
    /// IRQ 11.
    Periph3,
    /// IRQ 12.
    Mouse,
    /// IRQ 13.
    Fpu,
    /// IRQ 14.
    Ata1,
    /// IRQ 15.
    Ata2,
}

bitflags! {
    /// A set of IRQs.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct Irqs: u16 {
        /// IRQ 0.
        const TIMER = 1 << 0;
        /// IRQ 1.
        const KEYBOARD = 1 << 1;
        /// IRQ 2.
        const CASCADE = 1 << 2;
        /// IRQ 3.
        const COM2 = 1 << 3;
        /// IRQ 4.
        const COM1 = 1 << 4;
        /// IRQ 5.
        const LPT2 = 1 << 5;
        /// IRQ 6.
        const FLOPPY = 1 << 6;
        /// IRQ 7.
        const LPT1 = 1 << 7;
        /// IRQ 8.
        const REAL_TIME_CLOCK = 1 << 8;
        /// IRQ 9.
        const PERIPH1 = 1 << 9;
        /// IRQ 10.
        const PERIPH2 = 1 << 10;
        /// IRQ 11.
        const PERIPH3 = 1 << 11;
        /// IRQ 12.
        const MOUSE = 1 << 12;
        /// IRQ 13.
        const FPU = 1 << 13;
        /// IRQ 14.
        const ATA1 = 1 << 14;
        /// IRQ 15.
        const ATA2 = 1 << 15;
    }
}

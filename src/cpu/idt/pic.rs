use crate::drivers::{pic, ps2};
use crate::{printk, TERMINAL};

use super::InterruptStackFrame;

pub extern "x86-interrupt" fn timer(_stack_frame: InterruptStackFrame) {
    panic!("Received a TIMER interrupt (IRQ0).");
}

pub extern "x86-interrupt" fn keyboard(_stack_frame: InterruptStackFrame) {
    // Check the status register of the PS/2 controller. When the interrupt is received, the
    // output buffer should be full. It's probably not necessary to check, but it's probably
    // a good idea.
    if !ps2::status().intersects(ps2::PS2Status::OUTPUT_BUFFER_FULL) {
        printk!(
            "\
        	WARN: a keyboard interrupt was received (IRQ1), but the output buffer\n\
         	of the PS/2 controller is empty.\n\
            "
        );
        return;
    }

    // Send the scancode to the terminal.
    // Note: reading the scancode is *necessary* to clear the PS/2 controller's output buffer.
    // Without this, no new interrupts will be received.

    // TODO: buffer the scancode and process it in the main loop. Doing too much processing
    // in the IRQ handler will probably end up blocking the system.
    if !TERMINAL.lock().buffer_scancode(ps2::read_data()) {
        // The terminal buffer is full. We are probably lagging behind.
        printk!("WARN: the terminal buffer is full; we are dropping scancodes.\n");
    }

    pic::end_of_interrupt(pic::Irq::Keyboard);
}

pub extern "x86-interrupt" fn cascade(_stack_frame: InterruptStackFrame) {
    panic!("Received a CASCADE interrupt (IRQ2).");
}

pub extern "x86-interrupt" fn com2(_stack_frame: InterruptStackFrame) {
    panic!("Received a COM2 interrupt (IRQ3).");
}

pub extern "x86-interrupt" fn com1(_stack_frame: InterruptStackFrame) {
    panic!("Received a COM1 interrupt (IRQ4).");
}

pub extern "x86-interrupt" fn lpt2(_stack_frame: InterruptStackFrame) {
    panic!("Received a LPT2 interrupt (IRQ5).");
}

pub extern "x86-interrupt" fn floppy(_stack_frame: InterruptStackFrame) {
    panic!("Received a FLOPPY interrupt (IRQ6).");
}

pub extern "x86-interrupt" fn lpt1(_stack_frame: InterruptStackFrame) {
    panic!("Received a LPT1 interrupt (IRQ7).");
}

pub extern "x86-interrupt" fn rtc(_stack_frame: InterruptStackFrame) {
    panic!("Received a RTC interrupt (IRQ8).");
}

pub extern "x86-interrupt" fn periph1(_stack_frame: InterruptStackFrame) {
    panic!("Received a PERIPH1 interrupt (IRQ9).");
}

pub extern "x86-interrupt" fn periph2(_stack_frame: InterruptStackFrame) {
    panic!("Received a PERIPH2 interrupt (IRQ10).");
}

pub extern "x86-interrupt" fn periph3(_stack_frame: InterruptStackFrame) {
    panic!("Received a PERIPH3 interrupt (IRQ11).");
}

pub extern "x86-interrupt" fn mouse(_stack_frame: InterruptStackFrame) {
    panic!("Received a MOUSE interrupt (IRQ12).");
}

pub extern "x86-interrupt" fn fpu(_stack_frame: InterruptStackFrame) {
    panic!("Received a FPU interrupt (IRQ13).");
}

pub extern "x86-interrupt" fn ata1(_stack_frame: InterruptStackFrame) {
    panic!("Received a ATA1 interrupt (IRQ14).");
}

pub extern "x86-interrupt" fn ata2(_stack_frame: InterruptStackFrame) {
    panic!("Received a ATA2 interrupt (IRQ15).");
}

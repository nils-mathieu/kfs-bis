use core::arch::asm;

use crate::printk;

use super::InterruptStackFrame;

/// This interrupt service routine is called when the `int 0x80` instruction is executed
/// in user mode.
///
/// # Safety
///
/// This function expects to be called fro user mode by a user-space program.
#[naked]
pub unsafe extern "x86-interrupt" fn system_call(_stack_frame: InterruptStackFrame) {
    asm!(
        // Already on the stack, we have:
        //   ss, sp, flags, cs, ip
        //
        // The idea is to match the system call ABI of Linux, which is:
        "\
        call {}
        iretd
        ",
        sym inner,
        options(noreturn)
    );
}

/// The inner function of the system call handler.
extern "C" fn inner() {
    printk!("Received a system call interrupt!\n");
}

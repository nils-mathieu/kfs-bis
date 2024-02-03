//! Provides useful functions and other constructs.

mod critical_section;

pub mod instr;

pub use critical_section::RestoreInterrupts;

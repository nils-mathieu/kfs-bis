use core::mem::MaybeUninit;

use crate::utility::InitAllocator;

use super::UserId;

/// A list of processes.
pub struct Processes {
    /// The processes entries.
    ///
    /// Indices into the list are the IDs of the processes.
    processes: &'static mut [Option<Process>],
    /// The ID of the current process.
    ///
    /// This is *always* valid, as there is always a running process.
    current: ProcessId,
}

impl Processes {
    /// Creates a new [`Process`] instance.
    pub fn new(allocator: &mut InitAllocator, init: Process) -> Self {
        let processes = allocator.allocate_slice::<Option<Process>>(1024);
        for p in processes.iter_mut() {
            p.write(None);
        }

        let processes = unsafe { MaybeUninit::slice_assume_init_mut(processes) };
        processes[0] = Some(init);

        Self {
            processes,
            current: 0,
        }
    }
}

/// The ID of the process.
pub type ProcessId = u32;

/// This module contains information about a running process.
pub struct Process {
    /// The ID of the parent.
    pub parent: ProcessId,
    /// The signals that the process has eventually received.
    pub signals: Signals,
    /// The ID of the user that created the process.
    pub owner: UserId,
}

impl Process {
    /// Creates a new empty [`Process`] instance.
    pub fn new(parent: ProcessId, owner: UserId) -> Self {
        Self {
            parent,
            signals: Signals::default(),
            owner,
        }
    }
}

/// A list of received signal.
#[derive(Default)]
pub struct Signals {
    /// The list of signals that were received by the process.
    received: [Option<ReceivedSignal>; Signal::COUNT],
}

impl Signals {
    /// Schedules a signal to be handled by the process.
    ///
    /// If the process already has this signal type scheduled, this function
    /// returns `false`.
    #[must_use = "this method returns whether the signal was scheduled"]
    pub fn schedule(&mut self, signal: Signal, received_signal: ReceivedSignal) -> bool {
        let idx = signal as usize;

        if self.received[idx].is_some() {
            return false;
        }

        self.received[idx] = Some(received_signal);
        true
    }
}

/// Information about a received signal.
pub struct ReceivedSignal {
    /// The ID of the process that sent the signal.
    ///
    /// If the signal was sent by the kernel, this is [`None`].
    pub sent_by: Option<ProcessId>,
}

/// A signal.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Signal {
    /// The **SIGINT** signal.
    Int,
}

impl Signal {
    /// The number of signals.
    pub const COUNT: usize = 1;
}

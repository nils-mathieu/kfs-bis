use core::cell::UnsafeCell;
use core::convert::Infallible;
use core::mem::MaybeUninit;
use core::sync::atomic::AtomicU8;
use core::sync::atomic::Ordering::{Acquire, Relaxed, Release};

/// Allows initializing a value at a global scope.
pub struct OnceCell<T> {
    /// The current state of the cell.
    state: AtomicU8,
    /// The value of the cell.
    value: UnsafeCell<MaybeUninit<T>>,
}

unsafe impl<T: Send + Sync> Sync for OnceCell<T> {}
unsafe impl<T: Send> Send for OnceCell<T> {}

const UNINIT: u8 = 0;
const LOCKED: u8 = 1;
const INIT: u8 = 2;

impl<T> OnceCell<T> {
    /// Creates a new empty [`OnceCell<T>`].
    #[inline]
    pub const fn new() -> Self {
        Self {
            state: AtomicU8::new(UNINIT),
            value: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    /// Returns a reference to the inner initialized value.
    ///
    /// # Safety
    ///
    /// The inner value must be initialized.
    #[inline(always)]
    pub unsafe fn get_unchecked(&self) -> &T {
        unsafe { (*self.value.get()).assume_init_ref() }
    }

    /// Returns whether the [`OnceCell<T>`] is currently initialized.
    #[inline(always)]
    pub fn is_initialized(&self) -> bool {
        self.state.load(Acquire) == INIT
    }

    /// Returns the inner value of the [`OnceCell<T>`], if it is currently initialized.
    pub fn get(&self) -> Option<&T> {
        if self.is_initialized() {
            Some(unsafe { self.get_unchecked() })
        } else {
            None
        }
    }

    /// If the [`OnceCell<T>`] is currently initialized, the inner value is returned. Otherwise,
    /// the provided function is called to initialize the value.
    #[inline]
    pub fn get_or_try_init<E>(&self, f: impl FnOnce() -> Result<T, E>) -> Result<&T, E> {
        // Fast path: the cell is already initialized.
        if let Some(value) = self.get() {
            return Ok(value);
        }

        // Slow path: the cell is not initialized.
        self.get_or_try_init_cold(f)
    }

    #[cold]
    fn get_or_try_init_cold<E>(&self, f: impl FnOnce() -> Result<T, E>) -> Result<&T, E> {
        // Attempt to lock the cell to initialize the value.
        loop {
            match self
                .state
                .compare_exchange_weak(UNINIT, LOCKED, Acquire, Acquire)
            {
                Ok(_) => {
                    struct Guard<'a> {
                        /// The value to put back into the state.
                        to_restore: u8,
                        /// The reference to the state.
                        state: &'a AtomicU8,
                    }

                    impl<'a> Drop for Guard<'a> {
                        #[inline]
                        fn drop(&mut self) {
                            self.state.store(self.to_restore, Release);
                        }
                    }

                    // If the initialization fails (panic or error), we need to put the state back
                    // to `UNINIT`.
                    let mut guard = Guard {
                        to_restore: UNINIT,
                        state: &self.state,
                    };

                    // Initialize the value.
                    let value = f()?;
                    unsafe { (*self.value.get()).write(value) };

                    // The avlue was successfully initialized, we need to put the state to `INIT`
                    // upon returning.
                    guard.to_restore = INIT;

                    return Ok(unsafe { self.get_unchecked() });
                }
                Err(UNINIT) => {
                    // This is a spurious failure, we should retry.
                }
                Err(LOCKED) => {
                    // Another thread is currently initializing the value.
                    // We need to wait for it to finish.
                    while self.state.load(Relaxed) == LOCKED {}

                    match self.state.load(Acquire) {
                        INIT => {
                            // The value was initialized while we were waiting.
                            return Ok(unsafe { self.get_unchecked() });
                        }
                        UNINIT | LOCKED => {
                            // The other thread failed to initialize the value.
                            // We should retry.
                        }
                        _ => unsafe { core::hint::unreachable_unchecked() },
                    }
                }
                Err(INIT) => {
                    // The value was initialized while we were trying to lock the cell.
                    return Ok(unsafe { self.get_unchecked() });
                }
                _ => unsafe { core::hint::unreachable_unchecked() },
            }
        }
    }

    /// If the [`OnceCell<T>`] is currently initialized, the inner value is returned. Otherwise,
    /// the provided function is called to initialize the value.
    #[inline]
    pub fn get_or_init(&self, f: impl FnOnce() -> T) -> &T {
        match self.get_or_try_init(|| Ok::<T, Infallible>(f())) {
            Ok(ok) => ok,
            Err(err) => match err {},
        }
    }

    /// Initializes the [`OnceCell<T>`] with the provided value.
    ///
    /// If the cell is already initialized, the value is returned back to the caller.
    #[inline]
    pub fn set(&self, value: T) -> Result<(), T> {
        let mut slot = Some(value);
        self.get_or_init(|| unsafe { slot.take().unwrap_unchecked() });

        match slot {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }
}

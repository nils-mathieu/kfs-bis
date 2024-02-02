use core::cell::UnsafeCell;
use core::fmt::Debug;
use core::ops::{Deref, DerefMut};
#[cfg(debug_assertions)]
use core::panic::Location;
#[cfg(not(debug_assertions))]
use core::sync::atomic::AtomicBool;
#[cfg(debug_assertions)]
use core::sync::atomic::AtomicPtr;
use core::sync::atomic::Ordering::{Acquire, Relaxed, Release};

/// An error that might occur while attempting to lock a mutex.
pub struct CantLock {
    /// The location at which the mutex was locked.
    #[cfg(debug_assertions)]
    locked_at: &'static Location<'static>,
    /// The location at which the mutex *could not* be locked.
    #[cfg(debug_assertions)]
    attempt_at: &'static Location<'static>,

    /// Prevent the struct from being instantiated outside of this module.
    _private: (),
}

impl Debug for CantLock {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        #[cfg(debug_assertions)]
        {
            write!(
                f,
                "\
	            attempted to lock a mutex that was already being used\n\
	            - it was locked at {}\n\
	            - the attempt was made at {}\n\
	            ",
                self.attempt_at, self.locked_at
            )
        }

        #[cfg(not(debug_assertions))]
        {
            write!(f, "attempted to lock a mutex that was already being used")
        }
    }
}

/// A raw mutex implementation that stores the location at which the mutex was
/// locked.
#[cfg(debug_assertions)]
struct RawMutex(AtomicPtr<Location<'static>>);

#[cfg(debug_assertions)]
impl RawMutex {
    /// Creates a new [`RawMutex`] instance.
    #[inline]
    pub const fn new() -> Self {
        Self(AtomicPtr::new(core::ptr::null_mut()))
    }

    /// Attempts to lock the raw mutex.
    #[track_caller]
    #[inline]
    pub fn try_lock(&self) -> Result<(), CantLock> {
        let result = self.0.compare_exchange(
            core::ptr::null_mut(),
            Location::caller() as *const Location as *mut Location,
            Acquire,
            Relaxed,
        );

        match result {
            Ok(_) => Ok(()),
            Err(location) => Err(CantLock {
                locked_at: unsafe { &*location },
                attempt_at: Location::caller(),
                _private: (),
            }),
        }
    }

    /// Unlocks the mutex.
    ///
    /// # Safety
    ///
    /// The mutex must have been locked by the current context.
    #[inline(always)]
    pub unsafe fn unlock(&self) {
        self.0.store(core::ptr::null_mut(), Release);
    }
}

/// A raw mutex implementation that does not attempt to remember where it was locked.
#[cfg(not(debug_assertions))]
pub struct RawMutex(AtomicBool);

#[cfg(not(debug_assertions))]
impl RawMutex {
    /// Creates a new [`RawMutex`] instance.
    #[inline]
    pub const fn new() -> Self {
        Self(AtomicBool::new(false))
    }

    /// Attempt to lock the raw mutex.
    #[inline]
    pub fn try_lock(&self) -> Result<(), CantLock> {
        let result = self.0.compare_exchange(false, true, Acquire, Relaxed);

        if result.is_ok() {
            Ok(())
        } else {
            Err(CantLock { _private: () })
        }
    }

    /// Unlocks the mutex.
    ///
    /// # Safety
    ///
    /// The mutex must have been locked by the current context.
    #[inline(always)]
    pub unsafe fn unlock(&self) {
        self.0.store(false, Release);
    }
}

/// Represents a mutual exclusion primitive useful for protecting shared data.
///
/// This mutex implementation does not actually attempt to lock the data, and will
/// instead crash the system if the mutex is accessed twice at the same time.
pub struct Mutex<T: ?Sized> {
    /// The raw mutex implementation providing the locking mechanism.
    raw: RawMutex,
    /// The data protected by the mutex.
    value: UnsafeCell<T>,
}

unsafe impl<T: ?Sized + Send> Sync for Mutex<T> {}
unsafe impl<T: ?Sized + Send> Send for Mutex<T> {}

impl<T> Mutex<T> {
    /// Creates a new [`Mutex<T>`] instance.
    #[inline]
    pub const fn new(value: T) -> Self {
        Self {
            raw: RawMutex::new(),
            value: UnsafeCell::new(value),
        }
    }
}

impl<T: ?Sized> Mutex<T> {
    /// Attempts to lock the mutex and returns a guard if it succeeded.
    #[track_caller]
    #[inline]
    pub fn try_lock(&self) -> Result<MutexGuard<T>, CantLock> {
        self.raw.try_lock().map(|()| MutexGuard {
            raw: &self.raw,
            value: unsafe { &mut *self.value.get() },
        })
    }

    /// Locks the mutex and returns a guard that releases the lock when dropped.
    ///
    /// # Panics
    ///
    /// This function panics if the mutex is already locked.
    #[inline]
    #[track_caller]
    pub fn lock(&self) -> MutexGuard<T> {
        self.try_lock().unwrap()
    }
}

/// A guard that automatically releases the lock of a [`Mutex<T>`] when dropped.
pub struct MutexGuard<'a, T: ?Sized> {
    /// The raw mutex to unlock once the guard is dropped.
    raw: &'a RawMutex,
    /// The value protected by the lock.
    value: &'a mut T,
}

impl<T: ?Sized> Deref for MutexGuard<'_, T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<T: ?Sized> DerefMut for MutexGuard<'_, T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
    }
}

impl<T: ?Sized> Drop for MutexGuard<'_, T> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            self.raw.unlock();
        }
    }
}

impl<T: ?Sized> AsRef<T> for MutexGuard<'_, T> {
    #[inline(always)]
    fn as_ref(&self) -> &T {
        self.value
    }
}

impl<T: ?Sized> AsMut<T> for MutexGuard<'_, T> {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut T {
        self.value
    }
}

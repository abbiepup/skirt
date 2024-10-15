use crate::sync::atomic::{AtomicU8, Ordering};
use core::fmt::{Debug, Formatter};

pub struct Once {
    state: AtomicU8,
}

impl Default for Once {
    fn default() -> Self {
        Self::new()
    }
}

impl Once {
    const INCOMPLETE: u8 = 0;
    const RUNNING: u8 = 1;
    const COMPLETE: u8 = 2;

    /// Creates a new `Once` value.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            state: AtomicU8::new(Self::INCOMPLETE),
        }
    }

    /// Performs an initialization routine once and only once.
    /// The given closure will be executed if this is the first time `call_once` has been called, and otherwise the routine will not be invoked.
    pub fn call_once<F>(&self, f: F)
    where
        F: FnOnce(),
    {
        if self.is_completed() {
            return;
        }

        if self
            .state
            .compare_exchange(
                Self::INCOMPLETE,
                Self::RUNNING,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .is_ok()
        {
            f();

            self.state.store(Self::COMPLETE, Ordering::Release);
        } else {
            while self.state.load(Ordering::Acquire) == Self::RUNNING {
                #[cfg(not(feature = "std"))]
                core::hint::spin_loop();

                #[cfg(feature = "std")]
                std::thread::yield_now();
            }
        }
    }

    /// Returns true if some [`call_once()`] call has completed successfully.
    ///
    /// [`call_once()`]: Once::call_once
    ///
    /// # Examples
    ///
    /// ```
    /// use skirt::sync::Once;
    ///
    /// static INIT: Once = Once::new();
    ///
    /// assert_eq!(INIT.is_completed(), false);
    ///
    /// INIT.call_once(|| {
    ///     assert_eq!(INIT.is_completed(), false);
    /// });
    ///
    /// assert_eq!(INIT.is_completed(), true);
    /// ```
    pub fn is_completed(&self) -> bool {
        self.state.load(Ordering::Acquire) == Self::COMPLETE
    }
}

impl Debug for Once {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Once").finish_non_exhaustive()
    }
}
